use std::{collections::BTreeMap, sync::Arc, time::Duration};

use serde_json::{Value, json};
use tokio::{
    task::JoinSet,
    time::{Instant, sleep},
};

use crate::{
    cli::{self, TransportMode},
    probe::{self, Connection, Failure, Frame, Options, Probe},
};

pub const USAGE: &str = "Benchmark: mqtt-client bench conn|reconnect|echo [--clients 1..1000] [--hold-seconds 1..60] [--topic-prefix TOPICO --messages 1..100 (echo)] [opções de conexão]\n";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Conn,
    Reconnect,
    Echo,
}

struct Bench {
    mode: Mode,
    probe: Probe,
    clients: u16,
    hold: Duration,
    messages: u16,
    topic_prefix: Option<String>,
}

fn number<T>(
    args: &mut impl Iterator<Item = String>,
    range: std::ops::RangeInclusive<T>,
) -> Result<T, ()>
where
    T: PartialOrd + std::str::FromStr,
{
    args.next()
        .and_then(|value| value.parse().ok())
        .filter(|value| range.contains(value))
        .ok_or(())
}

fn parse(mode: &str, mut args: impl Iterator<Item = String>) -> Result<Bench, ()> {
    let mode = match mode {
        "conn" => Mode::Conn,
        "reconnect" => Mode::Reconnect,
        "echo" => Mode::Echo,
        _ => return Err(()),
    };
    let (mut clients, mut hold, mut messages, mut topic_prefix) = (None, None, None, None);
    let mut connection = Vec::new();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--clients" if clients.is_none() => clients = Some(number(&mut args, 1..=1000)?),
            "--hold-seconds" if hold.is_none() => hold = Some(number(&mut args, 1..=60)?),
            "--messages" if messages.is_none() && mode == Mode::Echo => {
                messages = Some(number(&mut args, 1..=100)?);
            }
            "--topic-prefix" if topic_prefix.is_none() && mode == Mode::Echo => {
                let value = args.next().ok_or(())?;
                if value.is_empty()
                    || value.len() > 960
                    || value.starts_with(['$', '-'])
                    || value.contains(['+', '#'])
                    || value.chars().any(char::is_control)
                {
                    return Err(());
                }
                topic_prefix = Some(value);
            }
            "--open-lab" | "--tls-password" | "--allow-remote-wss" => connection.push(flag),
            "--profile" | "--host" | "--port" | "--ws-url" | "--ca" | "--cert" | "--key"
            | "--username" | "--password-file" | "--protocol-version" | "--timeout-seconds" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or(())?;
                connection.extend([flag, value]);
            }
            _ => return Err(()),
        }
    }
    if mode == Mode::Echo && topic_prefix.is_none() {
        return Err(());
    }
    Ok(Bench {
        mode,
        probe: probe::parse(connection.into_iter())?,
        clients: clients.unwrap_or(1),
        hold: Duration::from_secs(hold.unwrap_or(1)),
        messages: messages.unwrap_or(1),
        topic_prefix,
    })
}

pub async fn run(mut args: impl Iterator<Item = String>) -> i32 {
    let Some(mode) = args.next() else {
        return invalid("invalid_options");
    };
    if matches!(mode.as_str(), "--help" | "-h") && args.next().is_none() {
        print!("{USAGE}");
        return 0;
    }
    let Ok(bench) = parse(&mode, args) else {
        return invalid("invalid_options");
    };
    let cli = &bench.probe.cli;
    let password = if cli.transport_mode == TransportMode::OpenLab {
        None
    } else {
        let Ok(password) = crate::read_mqtt_password(cli) else {
            return invalid("credentials_unavailable");
        };
        if cli::validate_password(&password).is_err() {
            return invalid("credentials_invalid");
        }
        Some(password)
    };
    let Ok(options) = Options::new(cli, password) else {
        return invalid("invalid_tls_materials");
    };
    let result = tokio::select! {
        biased;
        _ = crate::signals::shutdown() => None,
        result = run_waves(Arc::new(bench), Arc::new(options)) => Some(result),
    };
    if let Some((success, waves)) = result {
        println!(
            "{}",
            json!({"schema": 1, "operation": "bench", "mode": mode, "ok": success, "waves": waves})
        );
        i32::from(!success)
    } else {
        println!(
            "{}",
            json!({"schema": 1, "operation": "bench", "ok": false, "reason": "cancelled"})
        );
        1
    }
}

fn invalid(reason: &str) -> i32 {
    println!(
        "{}",
        json!({"schema": 1, "operation": "bench", "ok": false, "reason": reason})
    );
    2
}

#[derive(Default)]
struct Sample {
    connected_us: Option<u64>,
    echo_us: Vec<u64>,
    attempted: u16,
    failure: Option<Failure>,
}

async fn run_waves(bench: Arc<Bench>, options: Arc<Options>) -> (bool, Vec<Value>) {
    let mut waves = Vec::new();
    let mut success = true;
    for wave in 0..if bench.mode == Mode::Reconnect { 2 } else { 1 } {
        if wave > 0 {
            sleep(Duration::from_millis(500)).await;
        }
        let (ok, report) = run_wave(&bench, &options).await;
        success &= ok;
        waves.push(report);
    }
    (success, waves)
}

async fn run_wave(bench: &Arc<Bench>, options: &Arc<Options>) -> (bool, Value) {
    let start = Instant::now();
    let seconds = u32::from(bench.probe.cli.timeout_seconds);
    let seconds = seconds.min(bench.probe.cli.total_timeout_seconds.unwrap_or(seconds));
    let admission = start + Duration::from_secs(u64::from(seconds));
    let end = admission + bench.hold;
    let mut tasks = JoinSet::new();
    for index in 0..bench.clients {
        let bench = Arc::clone(bench);
        let options = Arc::clone(options);
        tasks.spawn(async move {
            let nonce = bench
                .probe
                .cli
                .client_id
                .strip_prefix("probe")
                .unwrap_or(&bench.probe.cli.client_id);
            let id = format!("b{nonce}-{index}");
            let mut connection = options.connection(&id);
            let mut sample = Sample::default();
            sample.failure = measure(&bench, &mut connection, &id, admission, end, &mut sample)
                .await
                .err();
            sample
        });
    }
    let (mut connect, mut echo, mut attempted) = (Vec::new(), Vec::new(), 0_u64);
    let mut failures = BTreeMap::new();
    let mut completed = 0_u16;
    while let Some(join) = tasks.join_next().await {
        if let Ok(sample) = join {
            connect.extend(sample.connected_us);
            echo.extend(sample.echo_us);
            attempted += u64::from(sample.attempted);
            if let Some(failure) = sample.failure {
                *failures
                    .entry(format!("{}/{}", failure.phase, failure.reason))
                    .or_insert(0_u16) += 1;
            } else {
                completed += 1;
            }
        } else {
            *failures.entry("worker/join".into()).or_insert(0_u16) += 1;
        }
    }
    let delivered = u32::try_from(echo.len()).unwrap_or(u32::MAX);
    let duration = Instant::now().saturating_duration_since(admission);
    let delivery_rate = (bench.mode == Mode::Echo && !duration.is_zero())
        .then(|| f64::from(delivered) / duration.as_secs_f64());
    (
        completed == bench.clients,
        json!({
            "clients_requested": bench.clients, "connected": connect.len(), "completed": completed,
            "failure_counts": failures, "elapsed_ms": micros(start.elapsed()) / 1000,
            "connect_us": percentiles(&mut connect), "echo_us": percentiles(&mut echo),
            "echo_attempted": attempted, "echo_delivered": delivered, "echo_per_second": delivery_rate,
        }),
    )
}

async fn measure(
    bench: &Bench,
    connection: &mut Connection,
    id: &str,
    admission: Instant,
    end: Instant,
    sample: &mut Sample,
) -> Result<(), Failure> {
    let start = Instant::now();
    let mut probe = Probe {
        cli: Arc::clone(&bench.probe.cli),
        topic: None,
        marker: None,
    };
    connection.wait_connect(&probe, admission).await?;
    sample.connected_us = Some(micros(start.elapsed()));
    if let Some(prefix) = &bench.topic_prefix {
        let topic = format!("{prefix}/{id}");
        connection.subscribe(&probe, &topic, admission).await?;
        poll_until(connection, &probe, admission).await?;
        probe.topic = Some(topic.clone());
        for index in 0..bench.messages {
            if Instant::now() >= end {
                return Err(Failure {
                    phase: "echo",
                    reason: "timeout",
                });
            }
            let marker = format!("{id}/{index}").into_bytes();
            connection.send_echo(&topic, &marker)?;
            sample.attempted += 1;
            probe.marker = Some(marker);
            let sent = Instant::now();
            while !matches!(connection.next(&probe, end, "echo").await?, Frame::Echo) {}
            sample.echo_us.push(micros(sent.elapsed()));
        }
    } else {
        poll_until(connection, &probe, end).await?;
    }
    Ok(())
}

async fn poll_until(
    connection: &mut Connection,
    probe: &Probe,
    deadline: Instant,
) -> Result<(), Failure> {
    loop {
        match connection.next(probe, deadline, "hold").await {
            Ok(_) => {}
            Err(Failure {
                reason: "timeout", ..
            }) if Instant::now() >= deadline => return Ok(()),
            Err(error) => return Err(error),
        }
    }
}

fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

fn percentiles(values: &mut [u64]) -> Value {
    if values.is_empty() {
        return Value::Null;
    }
    values.sort_unstable();
    let at = |percent: usize| values[(percent * values.len()).div_ceil(100) - 1];
    json!({"p50": at(50), "p95": at(95), "p99": at(99)})
}

#[cfg(test)]
mod tests {
    #[test]
    fn percentile_nearest_rank_handles_empty_and_extremes() {
        assert!(super::percentiles(&mut []).is_null());
        let mut values = [40, 10, 30, 20];
        let report = super::percentiles(&mut values);
        assert_eq!(report["p50"], 20);
        assert_eq!(report["p99"], 40);
    }
}
