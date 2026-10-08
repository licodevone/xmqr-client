use std::{env, path::PathBuf};

use rumqttc::QoS;
use rumqttc::mqttbytes::{valid_filter, valid_topic};

// Compatibility contract with XMQR broker: UTF-8 bytes, no normalization.
const MAX_TOPIC_BYTES: usize = 1024;
const MAX_MESSAGE_BYTES: usize = 4096;
const MAX_USERNAME_BYTES: usize = 128;
const MAX_PASSWORD_BYTES: usize = 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Pub { topic: String, message: String },
    Sub { filter: String, count: Option<u64> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMode {
    Secure,
    OpenLab,
    PlainAuthLab,
}

#[derive(Debug)]
pub struct Will {
    pub topic: String,
    pub message: String,
    pub qos: QoS,
    pub retain: bool,
}

#[derive(Debug)]
pub struct Cli {
    pub output: crate::output::Output,
    pub reconnect_attempts: u8,
    pub mode: Mode,
    pub will: Option<Will>,
    pub host: String,
    pub port: u16,
    pub ca: Option<PathBuf>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub client_id: String,
    pub username: Option<String>,
    pub password_file: Option<PathBuf>,
    pub qos: QoS,
    pub retain: bool,
    pub clean_session: bool,
    pub transport_mode: TransportMode,
}

pub const USAGE: &str = "Uso seguro (padrao):\n  mqtt-client pub --topic TOPICO --message TEXTO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n  mqtt-client sub --topic FILTRO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n\nLaboratorio aberto:\n  mqtt-client pub|sub --open-lab --topic TOPICO [opcoes]\nLaboratorio com senha em texto claro:\n  mqtt-client pub|sub --plain-auth-lab --topic TOPICO --username USUARIO [opcoes]\n\nWill: --will-topic TOPICO --will-message TEXTO [--will-qos 0|1|2] [--will-retain true|false].\nOpcoes: --output text|jsonl, --reconnect-attempts 0..10 (padrao 0; sub e conexao inicial pub), --message TEXTO (pub), --count N (sub), --host 127.0.0.1, --port 1883, --qos 0|1|2, --retain true|false, --clean-session true|false, --client-id ID.\nSem --password-file, os modos autenticados solicitam a senha sem exibi-la. Nunca informe senha na linha de comando.\nOs modos de laboratorio aceitam somente loopback. --plain-auth-lab envia usuario e senha sem criptografia. Nunca use fora de aula local.\n";

#[allow(clippy::too_many_lines)] // One pass enforces uniqueness and validates CLI modes.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut args = args.into_iter();
    let command = args.next().ok_or("informe pub ou sub")?;
    if command != "pub" && command != "sub" {
        return Err(format!("comando desconhecido: {command}"));
    }

    let mut output = None;
    let mut reconnect_attempts = None;
    let mut topic = None;
    let mut message = None;
    let mut count = None;
    let mut host = None;
    let mut port = None;
    let mut ca = None;
    let mut cert = None;
    let mut key = None;
    let mut client_id = None;
    let mut username = None;
    let mut password_file = None;
    let mut qos = None;
    let mut retain = None;
    let mut clean_session = None;
    let mut transport_mode = None;
    let mut will_topic = None;
    let mut will_message = None;
    let mut will_qos = None;
    let mut will_retain = None;

    while let Some(flag) = args.next() {
        if flag == "--open-lab" {
            if transport_mode.replace(TransportMode::OpenLab).is_some() {
                return Err("use somente um modo de laboratorio".into());
            }
            continue;
        }
        if flag == "--plain-auth-lab" {
            if transport_mode
                .replace(TransportMode::PlainAuthLab)
                .is_some()
            {
                return Err("use somente um modo de laboratorio".into());
            }
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("faltou valor para {flag}"))?;
        let slot = match flag.as_str() {
            "--output" => &mut output,
            "--reconnect-attempts" => &mut reconnect_attempts,
            "--topic" => &mut topic,
            "--will-topic" => &mut will_topic,
            "--will-message" => &mut will_message,
            "--will-qos" => &mut will_qos,
            "--will-retain" => &mut will_retain,
            "--message" => &mut message,
            "--count" => &mut count,
            "--host" => &mut host,
            "--port" => &mut port,
            "--ca" => &mut ca,
            "--cert" => &mut cert,
            "--key" => &mut key,
            "--client-id" => &mut client_id,
            "--username" => &mut username,
            "--password-file" => &mut password_file,
            "--qos" => &mut qos,
            "--retain" => &mut retain,
            "--clean-session" => &mut clean_session,
            _ => return Err(format!("opcao desconhecida: {flag}")),
        };
        if slot.replace(value).is_some() {
            return Err(format!("opcao repetida: {flag}"));
        }
    }

    let mode = parse_mode(&command, topic, message, count)?;
    let qos = match qos.as_deref().unwrap_or("0") {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => return Err("--qos deve ser 0, 1 ou 2".into()),
    };
    if command == "sub" && retain.is_some() {
        return Err("--retain somente vale para pub".into());
    }
    let retain = parse_bool(retain.as_deref().unwrap_or("false"), "--retain")?;
    let clean_session = parse_bool(
        clean_session.as_deref().unwrap_or("true"),
        "--clean-session",
    )?;

    let transport_mode = transport_mode.unwrap_or(TransportMode::Secure);
    let lab_mode = transport_mode != TransportMode::Secure;
    let host = host.unwrap_or_else(|| "127.0.0.1".into());
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err("--host invalido".into());
    }
    if lab_mode && host != "127.0.0.1" && host != "::1" {
        return Err("modos de laboratorio aceitam somente 127.0.0.1 ou ::1".into());
    }
    let port = port
        .map(|value| value.parse::<u16>().map_err(|_| "--port invalida"))
        .transpose()?
        .unwrap_or(if lab_mode { 1883 } else { 8883 });
    if port == 0 {
        return Err("--port deve ser maior que zero".into());
    }

    if transport_mode == TransportMode::OpenLab
        && (ca.is_some()
            || cert.is_some()
            || key.is_some()
            || username.is_some()
            || password_file.is_some())
    {
        return Err("--open-lab nao aceita certificados, usuario ou arquivo de senha".into());
    }
    if transport_mode == TransportMode::PlainAuthLab
        && (ca.is_some() || cert.is_some() || key.is_some())
    {
        return Err("--plain-auth-lab nao aceita certificados".into());
    }
    let secure_mode = !lab_mode;
    let ca = secure_mode
        .then(|| {
            ca.map(PathBuf::from)
                .or_else(|| env::var_os("MQTT_CA_CERT").map(PathBuf::from))
                .ok_or("informe --ca ou MQTT_CA_CERT")
        })
        .transpose()?;
    let cert = secure_mode
        .then(|| {
            cert.map(PathBuf::from)
                .or_else(|| env::var_os("MQTT_DEVICE_CERT").map(PathBuf::from))
                .ok_or("informe --cert ou MQTT_DEVICE_CERT")
        })
        .transpose()?;
    let key = secure_mode
        .then(|| {
            key.map(PathBuf::from)
                .or_else(|| env::var_os("MQTT_DEVICE_KEY").map(PathBuf::from))
                .ok_or("informe --key ou MQTT_DEVICE_KEY")
        })
        .transpose()?;
    let client_id = client_id.unwrap_or_else(|| format!("mqtt-{command}-{}", std::process::id()));
    if client_id.is_empty()
        || client_id.len() > 23
        || !client_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(
            "--client-id deve ter 1 a 23 caracteres ASCII alfanumericos, '-' ou '_'".into(),
        );
    }

    let username = if transport_mode == TransportMode::OpenLab {
        None
    } else {
        let username = username.ok_or("informe --username")?;
        if username.is_empty()
            || username.len() > MAX_USERNAME_BYTES
            || username.chars().any(char::is_control)
        {
            return Err("--username deve ter 1 a 128 bytes UTF-8, sem controles".into());
        }
        Some(username)
    };

    let will = match (will_topic, will_message) {
        (Some(topic), Some(message)) => {
            if topic.is_empty()
                || topic.len() > MAX_TOPIC_BYTES
                || topic.contains('\0')
                || !valid_topic(&topic)
            {
                return Err("--will-topic invalido ou acima de 1024 bytes".into());
            }
            if message.len() > MAX_MESSAGE_BYTES {
                return Err("--will-message acima de 4096 bytes".into());
            }
            let qos = match will_qos.as_deref().unwrap_or("0") {
                "0" => QoS::AtMostOnce,
                "1" => QoS::AtLeastOnce,
                "2" => QoS::ExactlyOnce,
                _ => return Err("--will-qos deve ser 0, 1 ou 2".into()),
            };
            let retain = parse_bool(will_retain.as_deref().unwrap_or("false"), "--will-retain")?;
            Some(Will {
                topic,
                message,
                qos,
                retain,
            })
        }
        (None, None) if will_qos.is_none() && will_retain.is_none() => None,
        _ => return Err("Will exige --will-topic e --will-message".into()),
    };
    let output = match output.as_deref().unwrap_or("text") {
        "text" => crate::output::Output::Text,
        "jsonl" => crate::output::Output::Jsonl,
        _ => return Err("--output deve ser text ou jsonl".into()),
    };
    let reconnect_attempts = reconnect_attempts
        .map(|value| {
            value
                .parse::<u8>()
                .map_err(|_| "--reconnect-attempts deve ser 0..10")
        })
        .transpose()?
        .unwrap_or(0);
    if reconnect_attempts > 10 {
        return Err("--reconnect-attempts deve ser 0..10".into());
    }
    Ok(Cli {
        output,
        reconnect_attempts,
        mode,
        will,
        host,
        port,
        ca,
        cert,
        key,
        client_id,
        username,
        password_file: password_file.map(PathBuf::from),
        qos,
        retain,
        clean_session,
        transport_mode,
    })
}

fn parse_bool(value: &str, flag: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{flag} deve ser true ou false")),
    }
}

/// Validates a secret without including it in diagnostics or logs.
pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.is_empty()
        || password.len() > MAX_PASSWORD_BYTES
        || password.chars().any(char::is_control)
    {
        return Err("senha MQTT deve ter 1 a 1024 bytes, sem caracteres de controle");
    }
    Ok(())
}

fn parse_mode(
    command: &str,
    topic: Option<String>,
    message: Option<String>,
    count: Option<String>,
) -> Result<Mode, String> {
    let topic = topic.ok_or("informe --topic")?;
    if topic.is_empty() || topic.len() > MAX_TOPIC_BYTES || topic.contains('\0') {
        return Err("topico/filtro vazio, longo demais ou com NUL".into());
    }

    if command == "pub" {
        if count.is_some() {
            return Err("--count somente vale para sub".into());
        }
        if !valid_topic(&topic) {
            return Err("topico de publicacao invalido".into());
        }
        let message = message.ok_or("informe --message")?;
        if message.len() > MAX_MESSAGE_BYTES {
            return Err(format!("mensagem excede {MAX_MESSAGE_BYTES} bytes"));
        }
        Ok(Mode::Pub { topic, message })
    } else {
        if message.is_some() {
            return Err("--message somente vale para pub".into());
        }
        if !valid_filter(&topic) {
            return Err("filtro de assinatura invalido".into());
        }
        let count = count
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| "--count deve ser inteiro positivo")
            })
            .transpose()?;
        if count == Some(0) {
            return Err("--count deve ser maior que zero".into());
        }
        Ok(Mode::Sub {
            filter: topic,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, TransportMode, parse, validate_password};

    fn base(mode: &str) -> Vec<String> {
        [
            mode,
            "--topic",
            "test/message",
            "--ca",
            "ca.crt",
            "--cert",
            "client.crt",
            "--key",
            "client.key",
            "--username",
            "lab-device",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    #[test]
    fn pub_requires_message_and_rejects_wildcards() {
        let mut args = base("pub");
        assert!(parse(args.clone()).is_err());
        args.extend(["--message".into(), "ola".into()]);
        assert!(matches!(
            parse(args.clone()).unwrap().mode,
            Mode::Pub { .. }
        ));
        args[2] = "test/#".into();
        assert!(parse(args).is_err());
    }

    #[test]
    fn sub_accepts_filter_and_bounded_count() {
        let mut args = base("sub");
        args[2] = "test/#".into();
        args.extend(["--count".into(), "2".into()]);
        assert!(matches!(
            parse(args.clone()).unwrap().mode,
            Mode::Sub { count: Some(2), .. }
        ));
        args[12] = "0".into();
        assert!(parse(args).is_err());
    }

    #[test]
    fn rejects_repeated_flags_and_oversized_message() {
        let mut args = base("pub");
        args.extend(["--message".into(), "x".repeat(4097)]);
        assert!(parse(args).is_err());
        let mut args = base("sub");
        args.extend(["--topic".into(), "other".into()]);
        assert!(parse(args).is_err());
    }

    #[test]
    fn requires_username_and_never_accepts_plaintext_password_argument() {
        let mut args = base("sub");
        args.truncate(args.len() - 2);
        assert!(parse(args).is_err());

        let mut args = base("sub");
        args.extend(["--password".into(), "secret".into()]);
        assert!(parse(args).is_err());
    }

    #[test]
    fn rejects_invalid_usernames_and_passwords_without_echoing_secret() {
        let mut args = base("sub");
        let last = args.len() - 1;
        args[last] = "a\0b".into();
        assert!(parse(args).is_err());
        assert!(validate_password("secret").is_ok());
        assert!(validate_password("").is_err());
        assert!(validate_password("a\nb").is_err());
        assert!(validate_password(&"a".repeat(1025)).is_err());
    }

    #[test]
    fn accepts_password_file_path_without_reading_secret_during_parse() {
        let mut args = base("sub");
        args.extend(["--password-file".into(), "/tmp/password".into()]);
        assert_eq!(
            parse(args).unwrap().password_file.unwrap(),
            std::path::PathBuf::from("/tmp/password")
        );
    }

    #[test]
    fn qos_retain_and_persistent_session_options_are_checked() {
        let mut args = base("pub");
        args.extend([
            "--message".into(),
            "state".into(),
            "--qos".into(),
            "2".into(),
            "--retain".into(),
            "true".into(),
            "--clean-session".into(),
            "false".into(),
            "--client-id".into(),
            "sensor-stable".into(),
        ]);
        let parsed = parse(args).unwrap();
        assert_eq!(parsed.qos, rumqttc::QoS::ExactlyOnce);
        assert!(parsed.retain);
        assert!(!parsed.clean_session);
        let mut bad = base("sub");
        bad.extend(["--retain".into(), "true".into()]);
        assert!(parse(bad).is_err());
        let mut bad = base("pub");
        bad.extend(["--message".into(), "x".into(), "--qos".into(), "3".into()]);
        assert!(parse(bad).is_err());
    }

    #[test]
    fn open_lab_is_explicit_anonymous_and_loopback_only() {
        let parsed = parse(
            [
                "sub",
                "--open-lab",
                "--topic",
                "test/message",
                "--client-id",
                "student-sub",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(parsed.transport_mode, TransportMode::OpenLab);
        assert_eq!(parsed.host, "127.0.0.1");
        assert_eq!(parsed.port, 1883);
        assert!(parsed.username.is_none());
        assert!(parsed.ca.is_none());

        let remote = [
            "sub",
            "--open-lab",
            "--topic",
            "test/message",
            "--host",
            "192.168.0.10",
        ]
        .into_iter()
        .map(str::to_owned);
        assert!(parse(remote).is_err());

        let ambiguous = [
            "sub",
            "--open-lab",
            "--topic",
            "test/message",
            "--username",
            "student",
        ]
        .into_iter()
        .map(str::to_owned);
        assert!(parse(ambiguous).is_err());
    }

    #[test]
    fn plain_auth_lab_requires_username_and_rejects_certificates() {
        let parsed = parse(
            [
                "sub",
                "--plain-auth-lab",
                "--topic",
                "test/message",
                "--username",
                "student",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(parsed.transport_mode, TransportMode::PlainAuthLab);
        assert_eq!(parsed.username.as_deref(), Some("student"));
        assert_eq!(parsed.port, 1883);
        assert!(parsed.ca.is_none());

        let no_username = ["sub", "--plain-auth-lab", "--topic", "test/message"]
            .into_iter()
            .map(str::to_owned);
        assert!(parse(no_username).is_err());

        let with_cert = [
            "sub",
            "--plain-auth-lab",
            "--topic",
            "test/message",
            "--username",
            "student",
            "--ca",
            "ca.crt",
        ]
        .into_iter()
        .map(str::to_owned);
        assert!(parse(with_cert).is_err());
    }

    #[test]
    fn pub_and_sub_use_shared_utf8_byte_limit() {
        for accepted in ["a".repeat(1024), "é".repeat(512), "🦀".repeat(256)] {
            for command in ["pub", "sub"] {
                let mut args = base(command);
                args[2] = accepted.clone();
                if command == "pub" {
                    args.extend(["--message".into(), "x".into()]);
                }
                assert!(parse(args.clone()).is_ok());
                args[2].push('a');
                assert!(parse(args).is_err());
            }
        }
        let mut args = base("sub");
        args[2] = format!("{}/+", "é".repeat(511));
        assert!(parse(args.clone()).is_ok());
        args[2].insert(0, 'a');
        assert!(parse(args).is_err());
    }

    #[test]
    fn will_options_validate_utf8_payload_qos_and_completeness() {
        let base = ["sub", "--open-lab", "--topic", "test/state"];
        let mut args: Vec<String> = base.iter().map(|s| (*s).to_owned()).collect();
        args.extend([
            "--will-topic".into(),
            "é".repeat(512),
            "--will-message".into(),
            String::new(),
            "--will-qos".into(),
            "2".into(),
            "--will-retain".into(),
            "true".into(),
        ]);
        let cli = parse(args.clone()).unwrap();
        assert!(cli.will.unwrap().retain);
        args[5].push('x');
        assert!(parse(args).is_err());
        for tail in [
            vec!["--will-qos", "1"],
            vec!["--will-topic", "test/+"],
            vec![
                "--will-topic",
                "test/state",
                "--will-message",
                "bye",
                "--will-qos",
                "3",
            ],
        ] {
            assert!(parse(base.iter().chain(tail.iter()).map(|s| (*s).to_owned())).is_err());
        }
    }
}

#[cfg(test)]
mod new_options_tests {
    use super::parse;
    use crate::output::Output;

    #[test]
    fn output_and_retry_validation() {
        let base = ["sub", "--open-lab", "--topic", "test"];
        let cli = parse(base.map(str::to_owned)).unwrap();
        assert_eq!(cli.output, Output::Text);
        assert_eq!(cli.reconnect_attempts, 0);
        for value in ["0", "1", "10"] {
            let cli = parse(
                base.iter()
                    .copied()
                    .chain(["--output", "jsonl", "--reconnect-attempts", value])
                    .map(str::to_owned),
            )
            .unwrap();
            assert_eq!(cli.output, Output::Jsonl);
            assert_eq!(cli.reconnect_attempts.to_string(), value);
        }
        for tail in [
            vec!["--output", "json"],
            vec!["--output", "text", "--output", "jsonl"],
            vec!["--reconnect-attempts", "11"],
            vec!["--reconnect-attempts", "-1"],
            vec!["--reconnect-attempts", "256"],
            vec!["--reconnect-attempts", "x"],
            vec!["--reconnect-attempts", "1", "--reconnect-attempts", "2"],
        ] {
            assert!(parse(base.iter().copied().chain(tail).map(str::to_owned)).is_err());
        }
    }
}
