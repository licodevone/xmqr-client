mod cli;
mod credentials;
#[cfg(target_os = "linux")]
mod outbox;
mod output;
mod payload;
mod profile;
#[cfg(any(test, all(windows, feature = "windows-service-runtime")))]
mod service_config;
#[cfg(all(windows, feature = "windows-service-runtime"))]
mod service_output;
mod session;
mod signals;
mod tls;
mod v5_session;
#[cfg(windows)]
mod windows_private;
#[cfg(all(windows, feature = "windows-service-runtime"))]
mod windows_service;

fn user_error(error: &(dyn std::error::Error + Send + Sync + 'static)) -> String {
    if let Some(rumqttc::v5::ConnectionError::MqttState(
        rumqttc::v5::StateError::ServerDisconnect { reason_code, .. },
    )) = error.downcast_ref::<rumqttc::v5::ConnectionError>()
    {
        return format!(
            "broker encerrou conexao MQTT 5: {reason_code:?} (0x{:02X})",
            *reason_code as u8
        );
    }
    error.to_string()
}

fn main() {
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--service-config")
    {
        if std::env::args_os().len() != 3 {
            std::process::exit(2);
        }
        let success =
            run_service_config(std::path::Path::new(&std::env::args_os().nth(2).unwrap()));
        if !success {
            eprintln!("Erro: configuracao privada ou sessao de servico falhou");
        }
        std::process::exit(i32::from(!success));
    }
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if std::env::args_os().len() == 2
        && std::env::args_os()
            .nth(1)
            .is_some_and(|arg| arg == "--windows-service-capabilities")
    {
        println!("{{\"schema\":1,\"component\":\"client\",\"service_entrypoint\":true}}");
        return;
    }
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "--windows-service")
    {
        let args: Vec<_> = std::env::args_os().skip(2).collect();
        if args.len() != 4 || args[2] != "--config" || !std::path::Path::new(&args[3]).is_absolute()
        {
            eprintln!("use --windows-service NAME LOG_DIR --config ABSOLUTE_PATH");
            std::process::exit(2);
        }
        let Some(name) = args[0].to_str() else {
            std::process::exit(2);
        };
        if windows_service::dispatch(name.to_owned(), args[1].clone().into(), run_service).is_err()
        {
            eprintln!("Nao foi possivel iniciar o dispatcher do servico");
            std::process::exit(1);
        }
        return;
    }
    std::process::exit(run(std::env::args().skip(1).collect()));
}

#[cfg(all(windows, feature = "windows-service-runtime"))]
fn run_service() -> bool {
    let Some(path) = std::env::args_os().nth(5) else {
        return false;
    };
    run_service_config(std::path::Path::new(&path))
}

#[cfg(all(windows, feature = "windows-service-runtime"))]
fn run_service_config(path: &std::path::Path) -> bool {
    let document = match service_config::load(path) {
        Ok(arguments) => arguments,
        Err(error) => {
            windows_service::report_failure(&error);
            return false;
        }
    };
    let Ok(cli) = cli::parse(document.arguments.clone()) else {
        windows_service::report_failure(&std::io::Error::other("invalid configuration"));
        return false;
    };
    if service_config::validate(&cli, document.output_file.is_some()).is_err() {
        return false;
    }
    if let Some(path) = &document.output_file
        && let Err(error) = service_output::configure(path, document.output_max_bytes)
    {
        windows_service::report_failure(&error);
        return false;
    }
    run_service_cli(cli) == 0
}

#[cfg(all(windows, feature = "windows-service-runtime"))]
#[tokio::main]
async fn run_service_cli(cli: cli::Cli) -> i32 {
    run_cli(cli).await
}

fn usage() -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        cli::USAGE,
        cli::V5_CONNECT_USAGE,
        cli::V5_PUBLISH_USAGE,
        cli::V5_SUBSCRIPTION_USAGE,
        cli::AUTH_USAGE,
        cli::V5_WILL_USAGE,
        cli::OUTBOX_USAGE,
        cli::SUBSCRIPTION_STATE_USAGE,
        cli::WS_USAGE
    )
}

#[tokio::main]
async fn run(arguments: Vec<String>) -> i32 {
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "--help" | "-h") {
        print!("{}", usage());
        return 0;
    }

    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("mqtt-client {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    let cli = match cli::parse(arguments) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("Erro: {message}\n\n{}", usage());
            return 2;
        }
    };
    run_cli(cli).await
}

async fn run_cli(cli: cli::Cli) -> i32 {
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if windows_service::active()
        && service_config::validate(&cli, service_output::active()).is_err()
    {
        eprintln!(
            "Configuracao de servico exige entrada nao interativa e saida privada para assinatura"
        );
        return 2;
    }
    let publication = if let cli::Mode::Pub { message, .. } = &cli.mode {
        let Ok(mut input) = payload::Input::start(message.clone(), cli.line_mode) else {
            eprintln!("Erro: nao foi possivel preparar entrada de payload");
            return 2;
        };
        let first = tokio::select! {
            biased;
            signal = signals::shutdown() => {
                if signal.is_err() { return 1; }
                eprintln!("Cancelado por sinal de encerramento");
                return 0;
            },
            result = input.next() => match result {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return 0, // Empty line input does not connect or publish.
                Err(error) => { eprintln!("Erro: {error}"); return 2; }
            },
        };
        if cli.payload_format == Some(1) && std::str::from_utf8(&first).is_err() {
            eprintln!("Erro: --payload-format 1 exige payload UTF-8 valido");
            return 2;
        }
        Some(payload::Publication { input, first })
    } else {
        None
    };
    // Open-lab is deliberately anonymous; secure mode prompts before opening
    // the network connection and never places the password in argv.
    let password = if cli.transport_mode == cli::TransportMode::OpenLab {
        None
    } else {
        let password_result = if let Some(ref path) = cli.password_file {
            credentials::read_password_file(path)
        } else {
            rpassword::prompt_password("Senha MQTT: ")
        };
        let password = match password_result {
            Ok(password) => password,
            Err(error) => {
                #[cfg(all(windows, feature = "windows-service-runtime"))]
                windows_service::report_failure(&error);
                #[cfg(not(all(windows, feature = "windows-service-runtime")))]
                let _ = error;
                eprintln!("Erro: nao foi possivel ler a senha MQTT com seguranca");
                return 2;
            }
        };
        if let Err(message) = cli::validate_password(&password) {
            eprintln!("Erro: {message}");
            return 2;
        }
        Some(password)
    };
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if windows_service::running().is_err() {
        return 1;
    }
    if let Err(error) = session::run(cli, password, publication).await {
        #[cfg(all(windows, feature = "windows-service-runtime"))]
        windows_service::report_failure(error.as_ref());
        eprintln!("Erro: {}", user_error(error.as_ref()));
        return 1;
    }
    0
}

#[cfg(test)]
mod diagnostic_tests {
    use super::user_error;
    use rumqttc::v5::{ConnectionError, StateError, mqttbytes::v5::DisconnectReasonCode};

    #[test]
    fn server_disconnect_exposes_code_without_untrusted_reason_string() {
        let error = ConnectionError::MqttState(StateError::ServerDisconnect {
            reason_code: DisconnectReasonCode::PacketTooLarge,
            reason_string: Some("secret-fixture-content\nforged-status".into()),
        });
        let message = user_error(&error);
        assert!(message.contains("PacketTooLarge"));
        assert!(message.contains("0x95"));
        assert!(!message.contains("secret-fixture-content"));
        assert!(!message.contains("forged-status"));
        let local = std::io::Error::other("local-error");
        assert_eq!(user_error(&local), "local-error");
    }
}
