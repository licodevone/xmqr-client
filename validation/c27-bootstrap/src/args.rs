// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
// C27 contract reconstruction: newly authored implementation, not production source.
use rumqttc::{
    QoS,
    mqttbytes::{valid_filter, valid_topic},
};
use std::{collections::HashMap, ffi::OsString, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Secure,
    AnonymousLoopback,
    AuthenticatedLoopback,
}

#[derive(Debug)]
pub struct Testament {
    pub topic: String,
    pub text: String,
    pub qos: QoS,
    pub retained: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Text,
    JsonLines,
}

#[derive(Debug)]
pub struct Settings {
    pub publisher: bool,
    pub topic: String,
    pub text: Option<String>,
    pub count: Option<u64>,
    pub host: String,
    pub port: u16,
    pub id: String,
    pub user: Option<String>,
    pub password_path: Option<PathBuf>,
    pub ca: Option<PathBuf>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub link: Link,
    pub qos: QoS,
    pub retained: bool,
    pub clean: bool,
    pub testament: Option<Testament>,
    pub encoding: Encoding,
    pub retries: u8,
}

impl Settings {
    pub const fn json(&self) -> bool {
        matches!(self.encoding, Encoding::JsonLines)
    }
}

pub const HELP: &str = "Uso: mqtt-client pub|sub --topic TOPICO [opcoes]\nPub: --message TEXTO [--qos 0|1|2] [--retain true|false]\nSub: [--count N]\nRede: --host HOST --port PORT --client-id ID --clean-session true|false\nTLS: --ca CA --cert CERT --key KEY --username USUARIO\nLabs somente loopback: --open-lab ou --plain-auth-lab --username USUARIO\nSenha oculta; --password-file PATH somente Linux/0600. Sem senha em argv.\nWill: --will-topic T --will-message M [--will-qos 0|1|2] [--will-retain true|false]\nSaida: --output text|jsonl; --reconnect-attempts 0..10\n";

pub fn parse(input: impl IntoIterator<Item = String>) -> Result<Settings, String> {
    parse_with_env(input, |key| std::env::var_os(key))
}

fn bool_value(flags: &HashMap<String, String>, name: &str, fallback: bool) -> Result<bool, String> {
    match flags.get(name).map(String::as_str) {
        None => Ok(fallback),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(_) => Err(format!("{name} exige true|false")),
    }
}

fn qos_value(flags: &HashMap<String, String>, name: &str) -> Result<QoS, String> {
    match flags.get(name).map_or("0", String::as_str) {
        "0" => Ok(QoS::AtMostOnce),
        "1" => Ok(QoS::AtLeastOnce),
        "2" => Ok(QoS::ExactlyOnce),
        _ => Err(format!("{name} exige 0|1|2")),
    }
}

fn checked_topic(value: &str, filter: bool) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 1024
        || value.contains('\0')
        || !(if filter {
            valid_filter(value)
        } else {
            valid_topic(value)
        })
    {
        Err("topico/filtro invalido ou acima de 1024 bytes".into())
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_lines)] // Complete CLI contract is validated in one pass.
fn parse_with_env(
    input: impl IntoIterator<Item = String>,
    env: impl Fn(&str) -> Option<OsString>,
) -> Result<Settings, String> {
    let mut input = input.into_iter();
    let command = input.next().ok_or("informe pub ou sub")?;
    let publisher = match command.as_str() {
        "pub" => true,
        "sub" => false,
        _ => return Err("comando desconhecido".into()),
    };
    let value_flags = [
        "--topic",
        "--message",
        "--count",
        "--host",
        "--port",
        "--client-id",
        "--ca",
        "--cert",
        "--key",
        "--username",
        "--password-file",
        "--qos",
        "--retain",
        "--clean-session",
        "--will-topic",
        "--will-message",
        "--will-qos",
        "--will-retain",
        "--output",
        "--reconnect-attempts",
    ];
    let mut flags = HashMap::new();
    while let Some(flag) = input.next() {
        let value = if flag == "--open-lab" || flag == "--plain-auth-lab" {
            String::new()
        } else if value_flags.contains(&flag.as_str()) {
            input
                .next()
                .ok_or_else(|| format!("faltou valor para {flag}"))?
        } else {
            return Err(format!("opcao desconhecida: {flag}"));
        };
        if flags.insert(flag, value).is_some() {
            return Err("opcao repetida".into());
        }
    }
    let anonymous = flags.contains_key("--open-lab");
    let authenticated_lab = flags.contains_key("--plain-auth-lab");
    if anonymous && authenticated_lab {
        return Err("nao combine laboratorios".into());
    }
    let link = if anonymous {
        Link::AnonymousLoopback
    } else if authenticated_lab {
        Link::AuthenticatedLoopback
    } else {
        Link::Secure
    };
    let topic = flags.get("--topic").ok_or("informe --topic")?.clone();
    checked_topic(&topic, !publisher)?;
    let text = flags.get("--message").cloned();
    if publisher && text.is_none() {
        return Err("informe --message".into());
    }
    if !publisher && text.is_some() {
        return Err("--message somente pub".into());
    }
    if text.as_ref().is_some_and(|v| v.len() > 4096) {
        return Err("payload acima de 4096 bytes".into());
    }
    if publisher && flags.contains_key("--count") {
        return Err("--count somente sub".into());
    }
    let count = flags
        .get("--count")
        .map(|v| v.parse::<u64>().map_err(|_| "--count invalido"))
        .transpose()?;
    if count == Some(0) {
        return Err("--count deve ser positivo".into());
    }
    if !publisher && flags.contains_key("--retain") {
        return Err("--retain somente pub".into());
    }
    let retained = bool_value(&flags, "--retain", false)?;
    let clean = bool_value(&flags, "--clean-session", true)?;
    let host = flags
        .get("--host")
        .cloned()
        .unwrap_or_else(|| "127.0.0.1".into());
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err("host invalido".into());
    }
    if link != Link::Secure && host != "127.0.0.1" && host != "::1" {
        return Err("laboratorio somente loopback".into());
    }
    let port = flags
        .get("--port")
        .map(|v| v.parse::<u16>().map_err(|_| "porta invalida"))
        .transpose()?
        .unwrap_or(if link == Link::Secure { 8883 } else { 1883 });
    if port == 0 {
        return Err("porta deve ser positiva".into());
    }
    let id = flags
        .get("--client-id")
        .cloned()
        .unwrap_or_else(|| format!("mqtt-{command}-{}", std::process::id()));
    if id.is_empty()
        || id.len() > 23
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("client-id exige 1..23 ASCII alnum,-,_".into());
    }
    if anonymous
        && ["--username", "--password-file", "--ca", "--cert", "--key"]
            .iter()
            .any(|k| flags.contains_key(*k))
    {
        return Err("open-lab nao aceita credenciais/TLS".into());
    }
    if link != Link::Secure
        && ["--ca", "--cert", "--key"]
            .iter()
            .any(|k| flags.contains_key(*k))
    {
        return Err("laboratorio nao aceita flags TLS".into());
    }
    let user = if anonymous {
        None
    } else {
        let name = flags.get("--username").ok_or("informe --username")?.clone();
        if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
            return Err("username invalido".into());
        }
        Some(name)
    };
    let pem = |flag: &str, variable: &str| -> Result<Option<PathBuf>, String> {
        if link != Link::Secure {
            return Ok(None);
        }
        flags
            .get(flag)
            .map(PathBuf::from)
            .or_else(|| env(variable).map(PathBuf::from))
            .map(Some)
            .ok_or_else(|| format!("informe {flag} ou {variable}"))
    };
    let ca = pem("--ca", "MQTT_CA_CERT")?;
    let cert = pem("--cert", "MQTT_DEVICE_CERT")?;
    let key = pem("--key", "MQTT_DEVICE_KEY")?;
    let has_will = flags.contains_key("--will-topic")
        || flags.contains_key("--will-message")
        || flags.contains_key("--will-qos")
        || flags.contains_key("--will-retain");
    let testament = if has_will {
        let topic = flags
            .get("--will-topic")
            .ok_or("Will exige topic e message")?
            .clone();
        let text = flags
            .get("--will-message")
            .ok_or("Will exige topic e message")?
            .clone();
        checked_topic(&topic, false)?;
        if text.len() > 4096 {
            return Err("Will acima de 4096 bytes".into());
        }
        Some(Testament {
            topic,
            text,
            qos: qos_value(&flags, "--will-qos")?,
            retained: bool_value(&flags, "--will-retain", false)?,
        })
    } else {
        None
    };
    let json = match flags.get("--output").map_or("text", String::as_str) {
        "text" => false,
        "jsonl" => true,
        _ => return Err("--output exige text|jsonl".into()),
    };
    let retries = flags
        .get("--reconnect-attempts")
        .map(|v| v.parse::<u8>().map_err(|_| "retries invalidos"))
        .transpose()?
        .unwrap_or(0);
    if retries > 10 {
        return Err("retries acima de 10".into());
    }
    Ok(Settings {
        publisher,
        topic,
        text,
        count,
        host,
        port,
        id,
        user,
        password_path: flags.get("--password-file").map(PathBuf::from),
        ca,
        cert,
        key,
        link,
        qos: qos_value(&flags, "--qos")?,
        retained,
        clean,
        testament,
        encoding: if json {
            Encoding::JsonLines
        } else {
            Encoding::Text
        },
        retries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parsed(tail: &[&str]) -> Result<Settings, String> {
        parse(
            ["sub", "--open-lab", "--topic", "t"]
                .into_iter()
                .chain(tail.iter().copied())
                .map(str::to_owned),
        )
    }
    #[test]
    fn defaults_and_all_negative_options() {
        let s = parsed(&[]).unwrap();
        assert_eq!(
            (s.port, s.retries, s.json(), s.clean),
            (1883, 0, false, true)
        );
        for bad in [
            vec!["--qos", "3"],
            vec!["--output", "json"],
            vec!["--count", "0"],
            vec!["--reconnect-attempts", "11"],
            vec!["--reconnect-attempts", "-1"],
            vec!["--port", "0"],
            vec!["--host", "localhost"],
            vec!["--plain-auth-lab"],
            vec!["--retain", "false"],
            vec!["--message", "x"],
            vec!["--will-qos", "1"],
            vec!["--qos", "1", "--qos", "2"],
            vec!["--password", "must-not-echo"],
            vec!["--client-id", "bad id"],
            vec!["--insecure"],
            vec!["--output"],
        ] {
            let err = parsed(&bad).unwrap_err();
            assert!(!err.contains("must-not-echo"));
        }
        let s = parsed(&[
            "--count",
            "2",
            "--output",
            "jsonl",
            "--reconnect-attempts",
            "10",
            "--clean-session",
            "false",
            "--will-topic",
            "offline",
            "--will-message",
            "",
            "--will-qos",
            "2",
            "--will-retain",
            "true",
        ])
        .unwrap();
        assert!(s.json());
        assert_eq!(s.count, Some(2));
        assert_eq!(s.retries, 10);
        assert!(!s.clean);
        assert!(s.testament.unwrap().retained);
    }
    #[test]
    fn quotas_apply_to_utf8_bytes_not_characters() {
        for command in ["pub", "sub"] {
            for length in [1024, 1025] {
                let mut input = vec![
                    command.into(),
                    "--open-lab".into(),
                    "--topic".into(),
                    "a".repeat(length),
                ];
                if command == "pub" {
                    input.extend(["--message".into(), String::new()]);
                }
                assert_eq!(parse(input).is_ok(), length == 1024);
            }
        }
        assert!(checked_topic(&"é".repeat(512), false).is_ok());
        assert!(checked_topic(&"é".repeat(513), false).is_err());
        for len in [4096, 4097] {
            assert_eq!(
                parse(
                    ["pub", "--open-lab", "--topic", "t", "--message"]
                        .map(str::to_owned)
                        .into_iter()
                        .chain(["x".repeat(len)])
                )
                .is_ok(),
                len == 4096
            );
        }
    }
    #[test]
    fn secure_flags_override_env_and_labs_ignore_env() {
        let args =
            ["sub", "--topic", "t", "--username", "u", "--ca", "explicit"].map(str::to_owned);
        let s = parse_with_env(args, |_| Some(OsString::from("environment"))).unwrap();
        assert_eq!(s.ca.unwrap(), PathBuf::from("explicit"));
        assert_eq!(s.cert.unwrap(), PathBuf::from("environment"));
        assert_eq!(s.port, 8883);
        assert!(parsed(&["--username", "u"]).is_err());
        assert!(parsed(&["--ca", "x"]).is_err());
        assert!(
            parse(
                ["sub", "--plain-auth-lab", "--topic", "t", "--username", "u"].map(str::to_owned)
            )
            .is_ok()
        );
    }
}
