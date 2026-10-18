mod cli;
mod credentials;
#[cfg(target_os = "linux")]
mod outbox;
mod output;
mod payload;
mod profile;
mod session;
mod signals;
mod tls;
mod v5_session;

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

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "--help" | "-h") {
        print!(
            "{}{}{}{}{}{}{}",
            cli::USAGE,
            cli::V5_CONNECT_USAGE,
            cli::V5_PUBLISH_USAGE,
            cli::V5_SUBSCRIPTION_USAGE,
            cli::AUTH_USAGE,
            cli::V5_WILL_USAGE,
            cli::OUTBOX_USAGE
        );
        return;
    }

    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("mqtt-client {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let cli = match cli::parse(arguments) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!(
                "Erro: {message}\n\n{}{}{}{}{}{}{}",
                cli::USAGE,
                cli::V5_CONNECT_USAGE,
                cli::V5_PUBLISH_USAGE,
                cli::V5_SUBSCRIPTION_USAGE,
                cli::AUTH_USAGE,
                cli::V5_WILL_USAGE,
                cli::OUTBOX_USAGE
            );
            std::process::exit(2);
        }
    };
    let publication = if let cli::Mode::Pub { message, .. } = &cli.mode {
        let Ok(mut input) = payload::Input::start(message.clone(), cli.line_mode) else {
            eprintln!("Erro: nao foi possivel preparar entrada de payload");
            std::process::exit(2);
        };
        let first = tokio::select! {
            biased;
            signal = signals::shutdown() => {
                if signal.is_err() { std::process::exit(1); }
                eprintln!("Cancelado por sinal de encerramento");
                return;
            },
            result = input.next() => match result {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return, // Empty line input does not connect or publish.
                Err(error) => { eprintln!("Erro: {error}"); std::process::exit(2); }
            },
        };
        if cli.payload_format == Some(1) && std::str::from_utf8(&first).is_err() {
            eprintln!("Erro: --payload-format 1 exige payload UTF-8 valido");
            std::process::exit(2);
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
        let Ok(password) = password_result else {
            eprintln!("Erro: nao foi possivel ler a senha MQTT com seguranca");
            std::process::exit(2);
        };
        if let Err(message) = cli::validate_password(&password) {
            eprintln!("Erro: {message}");
            std::process::exit(2);
        }
        Some(password)
    };
    if let Err(error) = session::run(cli, password, publication).await {
        eprintln!("Erro: {}", user_error(error.as_ref()));
        std::process::exit(1);
    }
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
