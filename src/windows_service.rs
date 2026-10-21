// SPDX-License-Identifier: MIT
//! Opt-in SCM lifecycle. Kept local to each independent executable.
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::watch;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};

struct Context {
    name: String,
    logs: PathBuf,
    run: fn() -> bool,
}
static CONTEXT: OnceLock<Context> = OnceLock::new();
static STATUS: Mutex<Option<ServiceStatusHandle>> = Mutex::new(None);
static LOG_LOCK: Mutex<()> = Mutex::new(());
static STOP: LazyLock<watch::Sender<bool>> = LazyLock::new(|| watch::channel(false).0);
define_windows_service!(service_entry, service_main);

pub fn active() -> bool {
    CONTEXT.get().is_some()
}

pub fn report_failure(error: &(dyn std::error::Error + 'static)) {
    let mut cause = Some(error);
    for _ in 0..8 {
        let Some(current) = cause else {
            break;
        };
        if let Some(error) = current.downcast_ref::<io::Error>() {
            log(match error.kind() {
                io::ErrorKind::Unsupported => "failure: unsupported platform operation",
                io::ErrorKind::PermissionDenied => "failure: access denied",
                io::ErrorKind::NotFound => "failure: required file or directory missing",
                io::ErrorKind::InvalidInput | io::ErrorKind::InvalidData => {
                    "failure: invalid configuration or data"
                }
                io::ErrorKind::AddrInUse => "failure: listener address already in use",
                io::ErrorKind::TimedOut => "failure: operation timed out",
                _ => "failure: input/output operation",
            });
            return;
        }
        cause = current.source();
    }
    log("failure: application initialization or runtime");
}

pub fn dispatch(name: String, logs: PathBuf, run: fn() -> bool) -> io::Result<()> {
    if !name.starts_with("xmqr-")
        || name.len() > 80
        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        || !logs.is_absolute()
        || !logs.is_dir()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid service name or log directory",
        ));
    }
    CONTEXT
        .set(Context {
            name: name.clone(),
            logs,
            run,
        })
        .map_err(|_| io::Error::other("service already initialized"))?;
    service_dispatcher::start(name, service_entry).map_err(io::Error::other)
}

fn status(state: ServiceState, failed: bool) -> ServiceStatus {
    let pending = matches!(
        state,
        ServiceState::StartPending | ServiceState::StopPending
    );
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: if state == ServiceState::Running {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else {
            ServiceControlAccept::empty()
        },
        exit_code: ServiceExitCode::Win32(u32::from(failed)),
        checkpoint: u32::from(pending),
        wait_hint: if pending {
            Duration::from_secs(30)
        } else {
            Duration::ZERO
        },
        process_id: None,
    }
}

pub fn running() -> io::Result<()> {
    if !active() {
        return Ok(());
    }
    let guard = STATUS
        .lock()
        .map_err(|_| io::Error::other("service status unavailable"))?;
    if *STOP.borrow() {
        // A stop during initialization must still reach the broker's normal
        // shutdown barrier. The retained watch value wakes shutdown() even
        // when its receiver is created after this check.
        return Ok(());
    }
    if let Some(handle) = *guard {
        handle
            .set_service_status(status(ServiceState::Running, false))
            .map_err(io::Error::other)?;
    }
    drop(guard);
    log("running");
    Ok(())
}

pub async fn shutdown() -> io::Result<()> {
    if !active() {
        let mut interrupt = tokio::signal::windows::ctrl_break()?;
        return tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = interrupt.recv() => Ok(()),
        };
    }
    let mut receiver = STOP.subscribe();
    receiver
        .wait_for(|stopped| *stopped)
        .await
        .map(|_| ())
        .map_err(io::Error::other)
}

fn service_main(_arguments: Vec<OsString>) {
    let Some(context) = CONTEXT.get() else {
        return;
    };
    let handler = service_control_handler::register(&context.name, |event| match event {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            STOP.send_replace(true);
            if let Ok(guard) = STATUS.lock()
                && let Some(handle) = *guard
            {
                let _ = handle.set_service_status(status(ServiceState::StopPending, false));
            }
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    });
    let Ok(handle) = handler else {
        log("control handler failed");
        return;
    };
    {
        let Ok(mut guard) = STATUS.lock() else {
            return;
        };
        *guard = Some(handle);
        // Serialize the first status with stop handling so StartPending can
        // never overwrite a StopPending status reported by the handler.
        let state = if *STOP.borrow() {
            ServiceState::StopPending
        } else {
            ServiceState::StartPending
        };
        if handle.set_service_status(status(state, false)).is_err() {
            *guard = None;
            log("start status failed");
            return;
        }
    }
    log("starting");
    let success = std::panic::catch_unwind(context.run).unwrap_or(false);
    log(if success {
        "stopped"
    } else {
        "failed: initialization or runtime; check platform prerequisites and configuration"
    });
    if let Ok(mut guard) = STATUS.lock() {
        let _ = handle.set_service_status(status(ServiceState::Stopped, !success));
        *guard = None;
    }
}

fn log(message: &str) {
    let Some(context) = CONTEXT.get() else {
        return;
    };
    let path = context.logs.join("service.log");
    let Ok(_guard) = LOG_LOCK.lock() else {
        return;
    };
    // Only fixed lifecycle messages, never config values, credentials or payloads.
    let _ = append_log(&path, message);
}

fn append_log(path: &Path, message: &str) -> io::Result<()> {
    if fs::metadata(path).is_ok_and(|m| m.len() > 1_048_576) {
        let old = path.with_extension("log.old");
        if old.exists() {
            fs::remove_file(&old)?;
        }
        fs::rename(path, old)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    writeln!(file, "{seconds} {message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_states_have_deadlines_and_no_controls() {
        for state in [ServiceState::StartPending, ServiceState::StopPending] {
            let s = status(state, false);
            assert!(!s.wait_hint.is_zero());
            assert!(s.controls_accepted.is_empty());
            assert!(s.checkpoint > 0);
        }
        assert!(
            status(ServiceState::Running, false)
                .controls_accepted
                .contains(ServiceControlAccept::STOP)
        );
        assert_eq!(
            status(ServiceState::Stopped, true).exit_code,
            ServiceExitCode::Win32(1)
        );
    }
}
