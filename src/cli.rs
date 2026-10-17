use std::{collections::HashSet, env, path::PathBuf};

use rumqttc::QoS;
use rumqttc::mqttbytes::{valid_filter, valid_topic};

// Compatibility contract with XMQR broker: UTF-8 bytes, no normalization.
const MAX_TOPIC_BYTES: usize = 1024;
const MAX_MESSAGE_BYTES: usize = 4096;
const MAX_USERNAME_BYTES: usize = 128;
const MAX_PASSWORD_BYTES: usize = 1024;
const MAX_SUBSCRIPTION_FILTERS: usize = 256;
const MAX_PACKET_BYTES: usize = 64 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Pub {
        topic: String,
        message: crate::payload::Source,
    },
    Sub {
        filters: Vec<String>,
        exclusions: Vec<String>,
        count: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMode {
    Secure,
    TlsPassword,
    OpenLab,
    PlainAuthLab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolVersion {
    Mqtt311,
    Mqtt5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStart {
    Clean,
    Resume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedFilter {
    All,
    Ignore,
    Only,
}

impl RetainedFilter {
    pub const fn accepts(self, retained: bool) -> bool {
        match self {
            Self::All => true,
            Self::Ignore => !retained,
            Self::Only => retained,
        }
    }
}

#[derive(Debug)]
pub struct Will {
    pub topic: String,
    pub message: Vec<u8>,
    pub qos: QoS,
    pub retain: bool,
    pub delay_seconds: Option<u32>,
}

#[derive(Debug)]
pub struct Cli {
    pub protocol_version: ProtocolVersion,
    pub request_problem_info: Option<u8>,
    pub request_response_info: Option<u8>,
    pub connect_user_properties: Vec<(String, String)>,
    pub line_mode: bool,
    pub retained_filter: RetainedFilter,
    pub output: crate::output::Output,
    pub reconnect_attempts: u8,
    pub repeat: u16,
    pub repeat_interval_ms: Option<u32>,
    pub keepalive_seconds: u16,
    pub timeout_seconds: u16,
    pub total_timeout_seconds: Option<u32>,
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
    pub clean_start: SessionStart,
    pub session_expiry: u32,
    pub message_expiry: Option<u32>,
    pub payload_format: Option<u8>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub publish_user_properties: Vec<(String, String)>,
    pub receive_maximum: Option<u16>,
    pub maximum_packet_size: Option<u32>,
    pub topic_alias_maximum: Option<u16>,
    pub transport_mode: TransportMode,
}

pub const USAGE: &str = "Uso seguro (padrao):\n  mqtt-client pub --topic TOPICO --message TEXTO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n  mqtt-client sub --topic FILTRO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n  mqtt-client pub|sub --profile PERFIL.toml [opcoes]\nTLS com senha, sem certificado de cliente:\n  mqtt-client pub|sub --tls-password --topic TOPICO --username USUARIO --ca CA.crt [opcoes]\n\nLaboratorio aberto:\n  mqtt-client pub|sub --open-lab --topic TOPICO [opcoes]\nLaboratorio com senha em texto claro:\n  mqtt-client pub|sub --plain-auth-lab --topic TOPICO --username USUARIO [opcoes]\n\nWill: --will-topic TOPICO --will-message TEXTO | --will-message-file PATH [--will-qos 0|1|2] [--will-retain true|false].\nOpcoes: --version (sozinho), --protocol-version 3.1.1|5 (padrao 3.1.1; MQTT5 inicial pub/sub QoS0/1/2), --message-file PATH | --stdin (pub, alternativos a --message), --line-mode (arquivo/stdin, LF/CRLF por mensagem), --output text|jsonl|hex|raw (hex/raw apenas sub; raw exige --count 1), --reconnect-attempts 0..10 (padrao 0; sub e conexao inicial pub), --keepalive-seconds 1..65535 (padrao 30), --timeout-seconds 1..300 (padrao 8; protocolo, nao espera de mensagens), --total-timeout-seconds 1..86400 (opcional; sessao apos payload inicial/senha, inclusive backoff), --message TEXTO (pub), --repeat 1..1000 --repeat-interval-ms 1..60000 (pub, fonte unica), --count N (sub, mensagens exibidas), --ignore-retained | --retained-only (sub), --exclude-topic FILTRO (sub, repetivel), --host 127.0.0.1, --port 1883, --qos 0|1|2, --retain true|false, --clean-session true|false, --client-id ID.\nSem --password-file, os modos autenticados solicitam a senha sem exibi-la. Nunca informe senha na linha de comando.\nOs modos de laboratorio aceitam somente loopback. --plain-auth-lab envia usuario e senha sem criptografia. Nunca use fora de aula local.\n";

pub const V5_CONNECT_USAGE: &str = "MQTT 5 CONNECT: --clean-start true|false, --session-expiry-seconds 0..4294967294|never, --receive-maximum 1..65535 (somente sub), --maximum-packet-size 5..65536 (somente sub), --topic-alias-maximum 0..16 (somente sub), --request-problem-info 0|1, --request-response-info 0|1, --connect-user-property CHAVE=VALOR (repetivel; ate 4096 bytes), --will-delay-seconds 0..4294967295 (exige Will). PUBLISH: --message-expiry-seconds 0..4294967295, --payload-format 0|1, --content-type TEXTO (ate 256 bytes; somente pub MQTT 5). Nao inclua segredos em argumentos.\n";

pub const V5_PUBLISH_USAGE: &str = "MQTT 5 PUBLISH adicional: --response-topic TOPICO (sem curingas), --correlation-data-hex HEX (ate 256 bytes), --publish-user-property CHAVE=VALOR (repetivel; ate 16 pares e 2048 bytes).\n";

#[allow(clippy::too_many_lines)] // One pass enforces uniqueness and validates CLI modes.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut args = crate::profile::expand(args.into_iter().collect())?.into_iter();
    let command = args.next().ok_or("informe pub ou sub")?;
    if command != "pub" && command != "sub" {
        return Err(format!("comando desconhecido: {command}"));
    }

    let mut output = None;
    let mut protocol_version = None;
    let mut request_problem_info = None;
    let mut request_response_info = None;
    let mut raw_connect_user_properties = Vec::new();
    let mut raw_publish_user_properties = Vec::new();
    let mut retained_filter = None;
    let mut reconnect_attempts = None;
    let mut repeat = None;
    let mut repeat_interval_ms = None;
    let mut keepalive_seconds = None;
    let mut timeout_seconds = None;
    let mut total_timeout_seconds = None;
    let mut message_file = None;
    let mut stdin = false;
    let mut line_mode = false;
    let mut topics = Vec::new();
    let mut exclusions = Vec::new();
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
    let mut clean_start = None;
    let mut session_expiry = None;
    let mut message_expiry = None;
    let mut payload_format = None;
    let mut content_type = None;
    let mut response_topic = None;
    let mut correlation_data_hex = None;
    let mut receive_maximum = None;
    let mut maximum_packet_size = None;
    let mut topic_alias_maximum = None;
    let mut transport_mode = None;
    let mut will_topic = None;
    let mut will_message = None;
    let mut will_message_file = None;
    let mut will_qos = None;
    let mut will_retain = None;
    let mut will_delay = None;

    while let Some(flag) = args.next() {
        if flag == "--ignore-retained" || flag == "--retained-only" {
            if command != "sub" {
                return Err("filtros retained somente valem para sub".into());
            }
            let selected = if flag == "--ignore-retained" {
                RetainedFilter::Ignore
            } else {
                RetainedFilter::Only
            };
            if retained_filter.replace(selected).is_some() {
                return Err("use somente um filtro retained, sem repetir".into());
            }
            continue;
        }
        if flag == "--stdin" || flag == "--line-mode" {
            let target = if flag == "--stdin" {
                &mut stdin
            } else {
                &mut line_mode
            };
            if std::mem::replace(target, true) {
                return Err(format!("opcao repetida: {flag}"));
            }
            continue;
        }
        if flag == "--open-lab" {
            if transport_mode.replace(TransportMode::OpenLab).is_some() {
                return Err("use somente um modo de laboratorio".into());
            }
            continue;
        }
        if flag == "--tls-password" {
            if transport_mode.replace(TransportMode::TlsPassword).is_some() {
                return Err("use somente um modo de transporte".into());
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
        if flag == "--topic" {
            topics.push(value);
            continue;
        }
        if flag == "--exclude-topic" {
            exclusions.push(value);
            continue;
        }
        if flag == "--connect-user-property" {
            raw_connect_user_properties.push(value);
            continue;
        }
        if flag == "--publish-user-property" {
            raw_publish_user_properties.push(value);
            continue;
        }
        let slot = match flag.as_str() {
            "--protocol-version" => &mut protocol_version,
            "--request-problem-info" => &mut request_problem_info,
            "--request-response-info" => &mut request_response_info,
            "--message-expiry-seconds" => &mut message_expiry,
            "--payload-format" => &mut payload_format,
            "--content-type" => &mut content_type,
            "--response-topic" => &mut response_topic,
            "--correlation-data-hex" => &mut correlation_data_hex,
            "--receive-maximum" => &mut receive_maximum,
            "--maximum-packet-size" => &mut maximum_packet_size,
            "--topic-alias-maximum" => &mut topic_alias_maximum,
            "--output" => &mut output,
            "--reconnect-attempts" => &mut reconnect_attempts,
            "--repeat" => &mut repeat,
            "--repeat-interval-ms" => &mut repeat_interval_ms,
            "--keepalive-seconds" => &mut keepalive_seconds,
            "--timeout-seconds" => &mut timeout_seconds,
            "--total-timeout-seconds" => &mut total_timeout_seconds,
            "--will-topic" => &mut will_topic,
            "--will-message" => &mut will_message,
            "--will-message-file" => &mut will_message_file,
            "--will-qos" => &mut will_qos,
            "--will-retain" => &mut will_retain,
            "--will-delay-seconds" => &mut will_delay,
            "--message-file" => &mut message_file,
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
            "--clean-start" => &mut clean_start,
            "--session-expiry-seconds" => &mut session_expiry,
            _ => return Err(format!("opcao desconhecida: {flag}")),
        };
        if slot.replace(value).is_some() {
            return Err(format!("opcao repetida: {flag}"));
        }
    }

    let mode = parse_mode(
        &command,
        topics,
        exclusions,
        message,
        message_file,
        stdin,
        line_mode,
        count,
    )?;
    let repeat_count = repeat
        .as_deref()
        .map(|value| {
            value
                .parse::<u16>()
                .ok()
                .filter(|count| (1..=1000).contains(count))
                .ok_or("--repeat deve ser 1..1000")
        })
        .transpose()?
        .unwrap_or(1);
    let interval = repeat_interval_ms
        .as_deref()
        .map(|value| {
            value
                .parse::<u32>()
                .ok()
                .filter(|ms| (1..=60_000).contains(ms))
                .ok_or("--repeat-interval-ms deve ser 1..60000")
        })
        .transpose()?;
    if repeat_count > 1 && interval.is_none() || repeat_count <= 1 && interval.is_some() {
        return Err("intervalo exige --repeat maior que 1".into());
    }
    if repeat.is_some() || interval.is_some() {
        match &mode {
            Mode::Pub {
                message: crate::payload::Source::Text(_) | crate::payload::Source::File(_),
                ..
            } if !line_mode => {}
            _ => return Err("repeticao exige pub de mensagem ou arquivo sem --line-mode".into()),
        }
    }
    let qos = match qos.as_deref().unwrap_or("0") {
        "0" => QoS::AtMostOnce,
        "1" => QoS::AtLeastOnce,
        "2" => QoS::ExactlyOnce,
        _ => return Err("--qos deve ser 0, 1 ou 2".into()),
    };
    let protocol_version = match protocol_version.as_deref().unwrap_or("3.1.1") {
        "3.1.1" => ProtocolVersion::Mqtt311,
        "5" => ProtocolVersion::Mqtt5,
        _ => return Err("--protocol-version deve ser 3.1.1 ou 5".into()),
    };
    if protocol_version != ProtocolVersion::Mqtt5
        && (request_problem_info.is_some()
            || request_response_info.is_some()
            || !raw_connect_user_properties.is_empty()
            || clean_start.is_some()
            || session_expiry.is_some())
    {
        return Err("propriedades CONNECT exigem --protocol-version 5".into());
    }
    if message_expiry.is_some() && (protocol_version != ProtocolVersion::Mqtt5 || command != "pub")
    {
        return Err("--message-expiry-seconds exige pub --protocol-version 5".into());
    }
    let message_expiry = message_expiry
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|_| "--message-expiry-seconds deve ser 0..4294967295".to_owned())
        })
        .transpose()?;
    if (payload_format.is_some()
        || content_type.is_some()
        || response_topic.is_some()
        || correlation_data_hex.is_some()
        || !raw_publish_user_properties.is_empty())
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "pub")
    {
        return Err("propriedades PUBLISH exigem pub --protocol-version 5".into());
    }
    let payload_format = payload_format
        .map(|raw| match raw.as_str() {
            "0" => Ok(0),
            "1" => Ok(1),
            _ => Err("--payload-format deve ser 0 ou 1".to_owned()),
        })
        .transpose()?;
    if content_type
        .as_ref()
        .is_some_and(|value| value.len() > 256 || !valid_mqtt5_property_text(value))
    {
        return Err("--content-type contem texto MQTT 5 invalido ou acima de 256 bytes".into());
    }
    if response_topic.as_ref().is_some_and(|value| {
        value.is_empty() || value.contains(['+', '#']) || !valid_mqtt5_property_text(value)
    }) {
        return Err("--response-topic exige topico MQTT 5 valido sem curingas".into());
    }
    let correlation_data = correlation_data_hex
        .map(|raw: String| {
            if raw.len() > 512 || !raw.len().is_multiple_of(2) || !raw.is_ascii() {
                return Err("--correlation-data-hex exige ate 256 bytes em hexadecimal".to_owned());
            }
            raw.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| {
                    let hex = std::str::from_utf8(pair).expect("ASCII verificado");
                    u8::from_str_radix(hex, 16)
                        .map_err(|_| "--correlation-data-hex exige bytes em hexadecimal".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let mut publish_user_properties = Vec::new();
    let mut publish_property_bytes = 0_usize;
    for raw in raw_publish_user_properties {
        let (key, value) = raw
            .split_once('=')
            .ok_or("--publish-user-property exige CHAVE=VALOR")?;
        if key.len() > 256
            || value.len() > 256
            || !valid_mqtt5_property_text(key)
            || !valid_mqtt5_property_text(value)
        {
            return Err("--publish-user-property contem texto MQTT 5 invalido".into());
        }
        publish_property_bytes = publish_property_bytes
            .checked_add(5 + key.len() + value.len())
            .ok_or("propriedades PUBLISH excedem 2048 bytes")?;
        if publish_user_properties.len() >= 16 || publish_property_bytes > 2048 {
            return Err("propriedades PUBLISH excedem limite de 16 pares ou 2048 bytes".into());
        }
        publish_user_properties.push((key.to_owned(), value.to_owned()));
    }
    if receive_maximum.is_some() && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--receive-maximum exige sub --protocol-version 5".into());
    }
    let receive_maximum = receive_maximum
        .map(|raw| {
            raw.parse::<u16>()
                .ok()
                .filter(|value| *value != 0)
                .ok_or("--receive-maximum deve ser 1..65535".to_owned())
        })
        .transpose()?;
    if maximum_packet_size.is_some()
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--maximum-packet-size exige sub --protocol-version 5".into());
    }
    let maximum_packet_size = maximum_packet_size
        .map(|raw| {
            raw.parse::<u32>()
                .ok()
                .filter(|value| (5..=65_536).contains(value))
                .ok_or("--maximum-packet-size deve ser 5..65536".to_owned())
        })
        .transpose()?;
    if topic_alias_maximum.is_some()
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--topic-alias-maximum exige sub --protocol-version 5".into());
    }
    let topic_alias_maximum = topic_alias_maximum
        .map(|raw| {
            raw.parse::<u16>()
                .ok()
                .filter(|value| *value <= 16)
                .ok_or("--topic-alias-maximum deve ser 0..16".to_owned())
        })
        .transpose()?;
    let parse_request_flag = |raw: Option<String>, flag: &str| -> Result<Option<u8>, String> {
        raw.map(|value| match value.as_str() {
            "0" => Ok(0),
            "1" => Ok(1),
            _ => Err(format!("{flag} deve ser 0 ou 1")),
        })
        .transpose()
    };
    let request_problem_info = parse_request_flag(request_problem_info, "--request-problem-info")?;
    let request_response_info =
        parse_request_flag(request_response_info, "--request-response-info")?;
    let mut connect_user_properties = Vec::new();
    let mut connect_property_bytes = 0_usize;
    for raw in raw_connect_user_properties {
        let (key, value) = raw
            .split_once('=')
            .ok_or("--connect-user-property exige CHAVE=VALOR")?;
        if !valid_mqtt5_property_text(key) || !valid_mqtt5_property_text(value) {
            return Err(
                "--connect-user-property contem texto MQTT 5 invalido ou acima de 1024 bytes"
                    .into(),
            );
        }
        connect_property_bytes = connect_property_bytes
            .checked_add(5 + key.len() + value.len())
            .ok_or("propriedades CONNECT excedem 4096 bytes")?;
        if connect_property_bytes > 4096 {
            return Err("propriedades CONNECT excedem 4096 bytes".into());
        }
        connect_user_properties.push((key.to_owned(), value.to_owned()));
    }
    if protocol_version == ProtocolVersion::Mqtt5
        && let Mode::Sub { filters, .. } = &mode
        && subscribe_packet_size(filters).is_none_or(|size| size + 1 > MAX_PACKET_BYTES)
    {
        return Err("pacote SUBSCRIBE MQTT 5 excede 64 KiB completos".into());
    }
    if command == "sub" && retain.is_some() {
        return Err("--retain somente vale para pub".into());
    }
    let retain = parse_bool(retain.as_deref().unwrap_or("false"), "--retain")?;
    if protocol_version == ProtocolVersion::Mqtt5 && clean_session.is_some() {
        return Err("--clean-session pertence a MQTT 3.1.1; use --clean-start".into());
    }
    let clean_session = parse_bool(
        clean_session.as_deref().unwrap_or("true"),
        "--clean-session",
    )?;
    let clean_start = if parse_bool(clean_start.as_deref().unwrap_or("true"), "--clean-start")? {
        SessionStart::Clean
    } else {
        SessionStart::Resume
    };
    let session_expiry = match session_expiry.as_deref().unwrap_or("0") {
        "never" => u32::MAX,
        raw => raw
            .parse::<u32>()
            .ok()
            .filter(|value| *value != u32::MAX)
            .ok_or("--session-expiry-seconds deve ser 0..4294967294 ou never")?,
    };

    let transport_mode = transport_mode.unwrap_or(TransportMode::Secure);
    let lab_mode = matches!(
        transport_mode,
        TransportMode::OpenLab | TransportMode::PlainAuthLab
    );
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
    if transport_mode == TransportMode::TlsPassword
        && (cert.is_some()
            || key.is_some()
            || env::var_os("MQTT_DEVICE_CERT").is_some()
            || env::var_os("MQTT_DEVICE_KEY").is_some())
    {
        return Err("--tls-password nao aceita certificado ou chave de cliente".into());
    }
    let secure_mode = !lab_mode;
    let ca = secure_mode
        .then(|| {
            ca.map(PathBuf::from)
                .or_else(|| env::var_os("MQTT_CA_CERT").map(PathBuf::from))
                .ok_or("informe --ca ou MQTT_CA_CERT")
        })
        .transpose()?;
    let cert = (transport_mode == TransportMode::Secure)
        .then(|| {
            cert.map(PathBuf::from)
                .or_else(|| env::var_os("MQTT_DEVICE_CERT").map(PathBuf::from))
                .ok_or("informe --cert ou MQTT_DEVICE_CERT")
        })
        .transpose()?;
    let key = (transport_mode == TransportMode::Secure)
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

    let will_from_file = will_message_file.is_some();
    if will_delay.is_some() && protocol_version != ProtocolVersion::Mqtt5 {
        return Err("--will-delay-seconds exige --protocol-version 5".into());
    }
    let will_delay = will_delay
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|_| "--will-delay-seconds deve ser 0..4294967295".to_owned())
        })
        .transpose()?;
    let will = match (will_topic, will_message, will_message_file) {
        (Some(topic), Some(message), None) | (Some(topic), None, Some(message)) => {
            if topic.is_empty()
                || topic.len() > MAX_TOPIC_BYTES
                || topic.contains('\0')
                || !valid_topic(&topic)
            {
                return Err("--will-topic invalido ou acima de 1024 bytes".into());
            }
            let message = if will_from_file {
                crate::payload::read_will_file(std::path::Path::new(&message))
                    .map_err(str::to_owned)?
            } else {
                if message.len() > MAX_MESSAGE_BYTES {
                    return Err("--will-message acima de 4096 bytes".into());
                }
                message.into_bytes()
            };
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
                delay_seconds: will_delay,
            })
        }
        (None, None, None)
            if will_qos.is_none() && will_retain.is_none() && will_delay.is_none() =>
        {
            None
        }
        _ => return Err("Will exige --will-topic e uma unica fonte de mensagem".into()),
    };
    let output = match output.as_deref().unwrap_or("text") {
        "text" => crate::output::Output::Text,
        "jsonl" => crate::output::Output::Jsonl,
        "hex" => crate::output::Output::Hex,
        "raw" => crate::output::Output::Raw,
        _ => return Err("--output deve ser text, jsonl, hex ou raw".into()),
    };
    if matches!(
        output,
        crate::output::Output::Hex | crate::output::Output::Raw
    ) {
        match &mode {
            Mode::Sub { count: Some(1), .. } => {}
            Mode::Sub { .. } if output == crate::output::Output::Hex => {}
            _ => return Err("--output raw exige sub --count 1; hex exige sub".into()),
        }
    }
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
    let keepalive_seconds = parse_seconds(
        keepalive_seconds.as_deref(),
        "--keepalive-seconds",
        30,
        u16::MAX,
    )?;
    let timeout_seconds = parse_seconds(timeout_seconds.as_deref(), "--timeout-seconds", 8, 300)?;
    let total_timeout_seconds = total_timeout_seconds
        .map(|value| {
            value
                .parse::<u32>()
                .ok()
                .filter(|seconds| (1..=86_400).contains(seconds))
                .ok_or("--total-timeout-seconds deve ser 1..86400")
        })
        .transpose()?;
    if protocol_version == ProtocolVersion::Mqtt5
        && (line_mode || reconnect_attempts != 0 || keepalive_seconds < 5)
    {
        return Err("MQTT 5 exige ausencia de modo linha e reconexao; keepalive >= 5".into());
    }
    Ok(Cli {
        protocol_version,
        request_problem_info,
        request_response_info,
        connect_user_properties,
        line_mode,
        retained_filter: retained_filter.unwrap_or(RetainedFilter::All),
        output,
        reconnect_attempts,
        repeat: repeat_count,
        repeat_interval_ms: interval,
        keepalive_seconds,
        timeout_seconds,
        total_timeout_seconds,
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
        clean_start,
        session_expiry,
        message_expiry,
        payload_format,
        content_type,
        response_topic,
        correlation_data,
        publish_user_properties,
        receive_maximum,
        maximum_packet_size,
        topic_alias_maximum,
        transport_mode,
    })
}

fn parse_seconds(value: Option<&str>, flag: &str, default: u16, max: u16) -> Result<u16, String> {
    let Some(value) = value else {
        return Ok(default);
    };
    let seconds = value
        .parse::<u16>()
        .ok()
        .filter(|seconds| (1..=max).contains(seconds));
    seconds.ok_or_else(|| format!("{flag} deve ser 1..{max}"))
}

fn parse_bool(value: &str, flag: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{flag} deve ser true ou false")),
    }
}

fn valid_mqtt5_property_text(value: &str) -> bool {
    value.len() <= 1024
        && !value.chars().any(|ch| {
            let code = ch as u32;
            ch == '\0'
                || ch.is_control()
                || (0xfdd0..=0xfdef).contains(&code)
                || code & 0xffff == 0xfffe
                || code & 0xffff == 0xffff
        })
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

#[allow(clippy::too_many_arguments)] // Source exclusivity is checked together with mode.
fn parse_mode(
    command: &str,
    topics: Vec<String>,
    exclusions: Vec<String>,
    message: Option<String>,
    message_file: Option<String>,
    stdin: bool,
    line_mode: bool,
    count: Option<String>,
) -> Result<Mode, String> {
    if command == "pub" {
        if !exclusions.is_empty() {
            return Err("--exclude-topic somente vale para sub".into());
        }
        if topics.len() != 1 {
            return Err(if topics.is_empty() {
                "informe --topic"
            } else {
                "pub aceita somente um --topic"
            }
            .into());
        }
        if count.is_some() {
            return Err("--count somente vale para sub".into());
        }
        let topic = &topics[0];
        validate_topic(topic)?;
        if !valid_topic(topic) {
            return Err("topico de publicacao invalido".into());
        }
        if usize::from(message.is_some()) + usize::from(message_file.is_some()) + usize::from(stdin)
            != 1
        {
            return Err(
                "pub exige exatamente uma fonte: --message, --message-file ou --stdin".into(),
            );
        }
        let message = if let Some(text) = message {
            if text.len() > MAX_MESSAGE_BYTES {
                return Err(format!("mensagem excede {MAX_MESSAGE_BYTES} bytes"));
            }
            if line_mode {
                return Err("--line-mode exige arquivo ou stdin".into());
            }
            crate::payload::Source::Text(text)
        } else if let Some(path) = message_file {
            crate::payload::Source::File(PathBuf::from(path))
        } else {
            crate::payload::Source::Stdin
        };
        Ok(Mode::Pub {
            topic: topic.clone(),
            message,
        })
    } else {
        if topics.is_empty() {
            return Err("informe ao menos um --topic para sub".into());
        }
        if topics.len() > MAX_SUBSCRIPTION_FILTERS {
            return Err(format!(
                "sub aceita no máximo {MAX_SUBSCRIPTION_FILTERS} filtros"
            ));
        }
        if message.is_some() || message_file.is_some() || stdin || line_mode {
            return Err("fontes de payload e --line-mode somente valem para pub".into());
        }
        let mut unique = HashSet::with_capacity(topics.len());
        for filter in &topics {
            validate_topic(filter)?;
            if !valid_filter(filter) {
                return Err("filtro de assinatura invalido".into());
            }
            if !unique.insert(filter) {
                return Err("filtros de assinatura duplicados".into());
            }
        }
        validate_exclusions(&exclusions)?;
        let packet_size =
            subscribe_packet_size(&topics).ok_or("pacote SUBSCRIBE excede o limite de tamanho")?;
        if packet_size > MAX_PACKET_BYTES {
            return Err("pacote SUBSCRIBE excede 64 KiB completos (cabecalho e corpo)".into());
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
            filters: topics,
            exclusions,
            count,
        })
    }
}

fn validate_exclusions(exclusions: &[String]) -> Result<(), String> {
    if exclusions.len() > MAX_SUBSCRIPTION_FILTERS {
        return Err(format!(
            "sub aceita no maximo {MAX_SUBSCRIPTION_FILTERS} exclusoes"
        ));
    }
    let mut unique = HashSet::with_capacity(exclusions.len());
    for filter in exclusions {
        validate_topic(filter)?;
        if !valid_filter(filter) {
            return Err("filtro de exclusao invalido".into());
        }
        if !unique.insert(filter) {
            return Err("filtros de exclusao duplicados".into());
        }
    }
    Ok(())
}

/// Match only validated MQTT topic filters and concrete topic names.
pub fn matches_filter(filter: &str, topic: &str) -> bool {
    if topic.starts_with('$') && matches!(filter.split('/').next(), Some("+" | "#")) {
        return false;
    }
    let mut filters = filter.split('/');
    let mut topics = topic.split('/');
    loop {
        match (filters.next(), topics.next()) {
            (Some("#"), _) | (None, None) => return true,
            (Some("+"), Some(_)) => {}
            (Some(expected), Some(actual)) if expected == actual => {}
            _ => return false,
        }
    }
}

fn validate_topic(topic: &str) -> Result<(), String> {
    if topic.is_empty() || topic.len() > MAX_TOPIC_BYTES || topic.contains('\0') {
        return Err("topico/filtro vazio, longo demais ou com NUL".into());
    }
    Ok(())
}

fn subscribe_packet_size(filters: &[String]) -> Option<usize> {
    let remaining = filters.iter().try_fold(2_usize, |size, filter| {
        size.checked_add(3)?.checked_add(filter.len())
    })?;
    let mut encoded_length_bytes = 1;
    let mut length = remaining;
    while length >= 128 {
        encoded_length_bytes += 1;
        length /= 128;
    }
    1_usize
        .checked_add(encoded_length_bytes)?
        .checked_add(remaining)
}

#[cfg(test)]
mod tests {
    #[test]
    fn mqtt5_will_delay_requires_will_and_version_five() {
        use super::parse;
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let good = parse(args(&[
            "--protocol-version",
            "5",
            "--will-topic",
            "last",
            "--will-message",
            "bye",
            "--will-delay-seconds",
            "2",
        ]))
        .unwrap();
        assert_eq!(good.will.unwrap().delay_seconds, Some(2));
        for extra in [
            vec!["--will-delay-seconds", "2"],
            vec!["--protocol-version", "5", "--will-delay-seconds", "2"],
            vec![
                "--protocol-version",
                "5",
                "--will-topic",
                "last",
                "--will-message",
                "bye",
                "--will-delay-seconds",
                "-1",
            ],
            vec![
                "--protocol-version",
                "5",
                "--will-topic",
                "last",
                "--will-message",
                "bye",
                "--will-delay-seconds",
                "4294967296",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
    }

    #[test]
    fn mqtt5_message_expiry_is_pub_only_and_bounded() {
        use super::parse;
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        for (raw, expected) in [("0", 0), ("42", 42), ("4294967295", u32::MAX)] {
            let cli = parse(args(&[
                "--protocol-version",
                "5",
                "--message-expiry-seconds",
                raw,
            ]))
            .unwrap();
            assert_eq!(cli.message_expiry, Some(expected));
        }
        for extra in [
            vec!["--message-expiry-seconds", "1"],
            vec!["--protocol-version", "5", "--message-expiry-seconds", "-1"],
            vec![
                "--protocol-version",
                "5",
                "--message-expiry-seconds",
                "4294967296",
            ],
            vec![
                "--protocol-version",
                "5",
                "--message-expiry-seconds",
                "1",
                "--message-expiry-seconds",
                "2",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        let sub = [
            "sub",
            "--open-lab",
            "--topic",
            "t",
            "--protocol-version",
            "5",
            "--message-expiry-seconds",
            "1",
        ];
        assert!(parse(sub.map(str::to_owned)).is_err());
    }

    #[test]
    fn mqtt5_publish_request_properties_validate_before_network() {
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let cli = parse(args(&[
            "--protocol-version",
            "5",
            "--response-topic",
            "reply/t",
            "--correlation-data-hex",
            "00ff07",
            "--publish-user-property",
            "trace=one",
            "--publish-user-property",
            "trace=two",
        ]))
        .unwrap();
        assert_eq!(cli.response_topic.as_deref(), Some("reply/t"));
        assert_eq!(
            cli.correlation_data.as_deref(),
            Some([0, 255, 7].as_slice())
        );
        assert_eq!(
            cli.publish_user_properties,
            [
                ("trace".into(), "one".into()),
                ("trace".into(), "two".into())
            ]
        );
        for extra in [
            vec!["--response-topic", "reply/t"],
            vec!["--protocol-version", "5", "--response-topic", "reply/+"],
            vec!["--protocol-version", "5", "--correlation-data-hex", "0"],
            vec!["--protocol-version", "5", "--correlation-data-hex", "gg"],
            vec![
                "--protocol-version",
                "5",
                "--publish-user-property",
                "missing",
            ],
            vec![
                "--protocol-version",
                "5",
                "--publish-user-property",
                "bad\0key=x",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        assert!(
            parse(args(&[
                "--protocol-version",
                "5",
                "--response-topic",
                &"r".repeat(1025)
            ]))
            .is_err()
        );
        assert!(
            parse(args(&[
                "--protocol-version",
                "5",
                "--correlation-data-hex",
                &"ff".repeat(257)
            ]))
            .is_err()
        );
        assert!(
            parse(args(&[
                "--protocol-version",
                "5",
                "--publish-user-property",
                &format!("k={}", "v".repeat(257))
            ]))
            .is_err()
        );
        let mut many = args(&["--protocol-version", "5"]);
        for _ in 0..17 {
            many.extend(["--publish-user-property".to_owned(), "k=v".to_owned()]);
        }
        assert!(parse(many).is_err());
        let sub = [
            "sub",
            "--open-lab",
            "--topic",
            "t",
            "--protocol-version",
            "5",
            "--response-topic",
            "reply/t",
        ];
        assert!(parse(sub.map(str::to_owned)).is_err());
    }

    #[test]
    fn mqtt5_publish_format_and_content_type_are_validated_before_network() {
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let cli = parse(args(&[
            "--protocol-version",
            "5",
            "--payload-format",
            "1",
            "--content-type",
            "text/plain",
        ]))
        .unwrap();
        assert_eq!(cli.payload_format, Some(1));
        assert_eq!(cli.content_type.as_deref(), Some("text/plain"));
        for extra in [
            vec!["--payload-format", "1"],
            vec!["--protocol-version", "5", "--payload-format", "2"],
            vec!["--protocol-version", "5", "--content-type", "bad\0type"],
            vec![
                "--protocol-version",
                "5",
                "--content-type",
                "a",
                "--content-type",
                "b",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        assert!(
            parse(
                [
                    "sub",
                    "--open-lab",
                    "--topic",
                    "t",
                    "--protocol-version",
                    "5",
                    "--content-type",
                    "text/plain",
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
        let too_long = "x".repeat(257);
        assert!(
            parse(args(&[
                "--protocol-version",
                "5",
                "--content-type",
                &too_long
            ]))
            .is_err()
        );
    }

    #[test]
    fn mqtt5_receive_maximum_is_sub_only_and_nonzero() {
        let base = ["sub", "--open-lab", "--topic", "t"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        for (raw, expected) in [("1", 1), ("65535", u16::MAX)] {
            let cli = parse(args(&["--protocol-version", "5", "--receive-maximum", raw])).unwrap();
            assert_eq!(cli.receive_maximum, Some(expected));
        }
        for extra in [
            vec!["--receive-maximum", "1"],
            vec!["--protocol-version", "5", "--receive-maximum", "0"],
            vec!["--protocol-version", "5", "--receive-maximum", "65536"],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        assert!(
            parse(
                [
                    "pub",
                    "--open-lab",
                    "--topic",
                    "t",
                    "--message",
                    "x",
                    "--protocol-version",
                    "5",
                    "--receive-maximum",
                    "1"
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn mqtt5_maximum_packet_size_is_sub_only_and_bounded() {
        let base = ["sub", "--open-lab", "--topic", "t"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        for (raw, expected) in [("5", 5), ("65536", 65_536)] {
            let cli = parse(args(&[
                "--protocol-version",
                "5",
                "--maximum-packet-size",
                raw,
            ]))
            .unwrap();
            assert_eq!(cli.maximum_packet_size, Some(expected));
        }
        for extra in [
            vec!["--maximum-packet-size", "100"],
            vec!["--protocol-version", "5", "--maximum-packet-size", "0"],
            vec!["--protocol-version", "5", "--maximum-packet-size", "4"],
            vec!["--protocol-version", "5", "--maximum-packet-size", "65537"],
            vec![
                "--protocol-version",
                "5",
                "--maximum-packet-size",
                "1",
                "--maximum-packet-size",
                "2",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        assert!(
            parse(
                [
                    "pub",
                    "--open-lab",
                    "--topic",
                    "t",
                    "--message",
                    "x",
                    "--protocol-version",
                    "5",
                    "--maximum-packet-size",
                    "100"
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn mqtt5_topic_alias_maximum_is_sub_only_and_bounded() {
        let base = ["sub", "--open-lab", "--topic", "t"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        for (raw, expected) in [("0", 0), ("1", 1), ("16", 16)] {
            let cli = parse(args(&[
                "--protocol-version",
                "5",
                "--topic-alias-maximum",
                raw,
            ]))
            .unwrap();
            assert_eq!(cli.topic_alias_maximum, Some(expected));
        }
        for extra in [
            vec!["--topic-alias-maximum", "1"],
            vec!["--protocol-version", "5", "--topic-alias-maximum", "17"],
            vec!["--protocol-version", "5", "--topic-alias-maximum", "-1"],
            vec!["--protocol-version", "5", "--topic-alias-maximum", "x"],
            vec![
                "--protocol-version",
                "5",
                "--topic-alias-maximum",
                "1",
                "--topic-alias-maximum",
                "2",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        assert!(
            parse(
                [
                    "pub",
                    "--open-lab",
                    "--topic",
                    "t",
                    "--message",
                    "x",
                    "--protocol-version",
                    "5",
                    "--topic-alias-maximum",
                    "1"
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn mqtt5_connect_properties_validate_and_preserve_pairs() {
        use super::parse;
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let arguments = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let parsed = parse(arguments(&[
            "--protocol-version",
            "5",
            "--request-problem-info",
            "0",
            "--request-response-info",
            "1",
            "--connect-user-property",
            "a=b",
            "--connect-user-property",
            "a=c",
        ]))
        .unwrap();
        assert_eq!(parsed.request_problem_info, Some(0));
        assert_eq!(parsed.request_response_info, Some(1));
        assert_eq!(
            parsed.connect_user_properties,
            [("a".into(), "b".into()), ("a".into(), "c".into())]
        );
        for extra in [
            vec!["--request-problem-info", "0"],
            vec!["--protocol-version", "5", "--request-problem-info", "2"],
            vec![
                "--protocol-version",
                "5",
                "--request-problem-info",
                "0",
                "--request-problem-info",
                "1",
            ],
            vec!["--protocol-version", "5", "--request-response-info", "2"],
            vec![
                "--protocol-version",
                "5",
                "--connect-user-property",
                "missing-equals",
            ],
            vec![
                "--protocol-version",
                "5",
                "--connect-user-property",
                "a=\u{0}",
            ],
        ] {
            assert!(parse(arguments(&extra)).is_err(), "{extra:?}");
        }
        let mut oversized = arguments(&["--protocol-version", "5", "--connect-user-property"]);
        oversized.push(format!("a={}", "x".repeat(1025)));
        assert!(parse(oversized).is_err());
        let mut total_oversized = arguments(&["--protocol-version", "5"]);
        for _ in 0..4 {
            total_oversized.extend([
                "--connect-user-property".to_owned(),
                format!("a={}", "x".repeat(1023)),
            ]);
        }
        assert!(parse(total_oversized).is_err());
    }

    #[test]
    fn mqtt5_initial_scope_is_explicit_and_validated_before_network() {
        use super::{Mode, ProtocolVersion, parse};
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let arguments = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            parse(arguments(&[])).unwrap().protocol_version,
            ProtocolVersion::Mqtt311
        );
        assert_eq!(
            parse(arguments(&["--protocol-version", "5", "--qos", "1"]))
                .unwrap()
                .protocol_version,
            ProtocolVersion::Mqtt5
        );
        let persistent = parse(arguments(&[
            "--protocol-version",
            "5",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "never",
        ]))
        .unwrap();
        assert_eq!(persistent.clean_start, super::SessionStart::Resume);
        assert_eq!(persistent.session_expiry, u32::MAX);
        assert_eq!(
            parse(arguments(&[
                "--protocol-version",
                "5",
                "--session-expiry-seconds",
                "1"
            ]))
            .unwrap()
            .session_expiry,
            1
        );
        assert_eq!(
            parse(arguments(&[
                "--protocol-version",
                "5",
                "--session-expiry-seconds",
                "4294967294"
            ]))
            .unwrap()
            .session_expiry,
            u32::MAX - 1
        );
        for extra in [
            vec!["--protocol-version", "6"],
            vec!["--protocol-version", "5", "--clean-session", "false"],
            vec!["--protocol-version", "5", "--reconnect-attempts", "1"],
            vec!["--clean-start", "false"],
            vec!["--session-expiry-seconds", "never"],
            vec![
                "--protocol-version",
                "5",
                "--session-expiry-seconds",
                "4294967296",
            ],
            vec![
                "--protocol-version",
                "5",
                "--session-expiry-seconds",
                "4294967295",
            ],
            vec!["--protocol-version", "5", "--session-expiry-seconds", "-1"],
            vec!["--protocol-version", "5", "--clean-start", "yes"],
            vec![
                "--protocol-version",
                "5",
                "--clean-start",
                "true",
                "--clean-start",
                "false",
            ],
        ] {
            assert!(parse(arguments(&extra)).is_err());
        }
        let sub = [
            "sub",
            "--open-lab",
            "--topic",
            "t",
            "--protocol-version",
            "5",
        ];
        assert!(matches!(
            parse(sub.into_iter().map(str::to_owned)).unwrap().mode,
            Mode::Sub { .. }
        ));
        assert!(matches!(
            parse(arguments(&["--protocol-version", "5"])).unwrap().mode,
            Mode::Pub { .. }
        ));
    }
    use super::{Mode, RetainedFilter, TransportMode, parse, validate_password};

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
    fn retained_filters_are_sub_only_and_mutually_exclusive() {
        assert_eq!(
            parse(base("sub")).unwrap().retained_filter,
            RetainedFilter::All
        );
        for (flag, expected) in [
            ("--ignore-retained", RetainedFilter::Ignore),
            ("--retained-only", RetainedFilter::Only),
        ] {
            let mut args = base("sub");
            args.push(flag.into());
            assert_eq!(parse(args.clone()).unwrap().retained_filter, expected);
            args.push(flag.into());
            assert!(parse(args).is_err());
            let mut args = base("pub");
            args.extend(["--message".into(), "test".into(), flag.into()]);
            assert!(parse(args).is_err());
        }
        let mut args = base("sub");
        args.extend(["--ignore-retained".into(), "--retained-only".into()]);
        assert!(parse(args).is_err());
    }

    #[test]
    fn timing_options_preserve_defaults_and_validate_bounds() {
        let defaults = parse(base("sub")).unwrap();
        assert_eq!(defaults.keepalive_seconds, 30);
        assert_eq!(defaults.timeout_seconds, 8);
        assert_eq!(defaults.total_timeout_seconds, None);
        for (flag, max) in [("--keepalive-seconds", 65535), ("--timeout-seconds", 300)] {
            for value in ["1".to_owned(), max.to_string()] {
                let mut args = base("sub");
                args.extend([flag.into(), value.clone()]);
                let cli = parse(args).unwrap();
                let actual = if flag == "--keepalive-seconds" {
                    cli.keepalive_seconds
                } else {
                    cli.timeout_seconds
                };
                assert_eq!(actual.to_string(), value);
            }
            for value in [
                "0".to_owned(),
                (max + 1).to_string(),
                "-1".into(),
                "text".into(),
                "1.5".into(),
            ] {
                let mut args = base("sub");
                args.extend([flag.into(), value]);
                assert!(parse(args).is_err());
            }
            let mut args = base("sub");
            args.push(flag.into());
            assert!(parse(args).is_err());
            let mut args = base("sub");
            args.extend([flag.into(), "1".into(), flag.into(), "2".into()]);
            assert!(parse(args).is_err());
        }
    }

    #[test]
    fn total_timeout_is_optional_bounded_and_unique() {
        for value in ["1", "86400"] {
            let mut args = base("sub");
            args.extend(["--total-timeout-seconds".into(), value.into()]);
            assert_eq!(
                parse(args.clone())
                    .unwrap()
                    .total_timeout_seconds
                    .unwrap()
                    .to_string(),
                value
            );
            args.extend(["--total-timeout-seconds".into(), value.into()]);
            assert!(parse(args).is_err());
        }
        for value in ["0", "86401", "-1", "1.5", "text", "4294967296"] {
            let mut args = base("sub");
            args.extend(["--total-timeout-seconds".into(), value.into()]);
            assert!(parse(args).is_err());
        }
        let mut args = base("sub");
        args.push("--total-timeout-seconds".into());
        assert!(parse(args).is_err());
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
    fn exclusions_are_validated_before_network_and_match_mqtt_levels() {
        let mut args = base("sub");
        args.extend(["--exclude-topic".into(), "test/+".into()]);
        assert!(matches!(parse(args.clone()).unwrap().mode,
            Mode::Sub { exclusions, .. } if exclusions == ["test/+" ]));
        for invalid in ["test/#/extra", "test+", "", "a\0b", &"é".repeat(513)] {
            let mut candidate = base("sub");
            candidate.extend(["--exclude-topic".into(), invalid.into()]);
            assert!(parse(candidate).is_err());
        }
        args.extend(["--exclude-topic".into(), "test/+".into()]);
        assert!(parse(args).is_err());
        let mut args = base("pub");
        args.extend([
            "--message".into(),
            "x".into(),
            "--exclude-topic".into(),
            "#".into(),
        ]);
        assert!(parse(args).is_err());
        let mut args = base("sub");
        for index in 0..257 {
            args.extend(["--exclude-topic".into(), format!("x/{index}")]);
        }
        assert!(parse(args).is_err());

        for (filter, topic, expected) in [
            ("a/+", "a/", true),
            ("a/#", "a", true),
            ("a/+", "a/b/c", false),
            ("#", "$SYS/status", false),
            ("+/status", "$SYS/status", false),
            ("$SYS/#", "$SYS/status", true),
            ("é/+", "é/valor", true),
        ] {
            assert_eq!(super::matches_filter(filter, topic), expected);
        }
    }

    #[test]
    fn repeat_is_finite_and_only_accepts_single_message_sources() {
        let mut args = base("pub");
        args.extend([
            "--message".into(),
            "x".into(),
            "--repeat".into(),
            "3".into(),
            "--repeat-interval-ms".into(),
            "25".into(),
        ]);
        let cli = parse(args).unwrap();
        assert_eq!((cli.repeat, cli.repeat_interval_ms), (3, Some(25)));
        let mut mqtt5 = base("pub");
        mqtt5.extend([
            "--message".into(),
            "x".into(),
            "--protocol-version".into(),
            "5".into(),
            "--repeat".into(),
            "3".into(),
            "--repeat-interval-ms".into(),
            "25".into(),
        ]);
        let cli = parse(mqtt5).unwrap();
        assert_eq!((cli.repeat, cli.repeat_interval_ms), (3, Some(25)));
        for tail in [
            vec!["--repeat", "0"],
            vec!["--repeat", "1001"],
            vec!["--repeat", "x"],
            vec!["--repeat", "2"],
            vec!["--repeat-interval-ms", "20"],
            vec!["--repeat", "2", "--repeat-interval-ms", "0"],
            vec!["--repeat", "2", "--repeat-interval-ms", "60001"],
        ] {
            let mut args = base("pub");
            args.extend(["--message".into(), "x".into()]);
            args.extend(tail.into_iter().map(str::to_owned));
            assert!(parse(args).is_err());
        }
        let mut args = base("sub");
        args.extend([
            "--repeat".into(),
            "2".into(),
            "--repeat-interval-ms".into(),
            "10".into(),
        ]);
        assert!(parse(args).is_err());
        let mut args = base("pub");
        args.extend([
            "--stdin".into(),
            "--repeat".into(),
            "2".into(),
            "--repeat-interval-ms".into(),
            "10".into(),
        ]);
        assert!(parse(args).is_err());
    }

    #[test]
    fn binary_output_modes_require_sub_and_raw_single_message() {
        let mut args = base("sub");
        args.extend(["--output".into(), "hex".into()]);
        assert_eq!(parse(args).unwrap().output, crate::output::Output::Hex);
        let mut args = base("sub");
        args.extend(["--output".into(), "raw".into()]);
        assert!(parse(args).is_err());
        let mut args = base("sub");
        args.extend([
            "--output".into(),
            "raw".into(),
            "--count".into(),
            "1".into(),
        ]);
        assert_eq!(parse(args).unwrap().output, crate::output::Output::Raw);
        let mut args = base("pub");
        args.extend([
            "--message".into(),
            "x".into(),
            "--output".into(),
            "hex".into(),
        ]);
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
        assert!(
            matches!(parse(args).unwrap().mode, Mode::Sub { filters, .. } if filters.len() == 2)
        );
        let mut args = base("sub");
        args.extend(["--topic".into(), "test/message".into()]);
        assert!(parse(args).is_err());
        let mut args = base("pub");
        args.extend([
            "--topic".into(),
            "other".into(),
            "--message".into(),
            "x".into(),
        ]);
        assert!(parse(args).is_err());
    }

    #[test]
    fn sub_filters_obey_count_byte_and_packet_limits() {
        let mut args = vec!["sub".into(), "--open-lab".into()];
        for index in 0..256 {
            args.extend(["--topic".into(), format!("topic/{index}")]);
        }
        assert!(
            matches!(parse(args.clone()).unwrap().mode, Mode::Sub { filters, .. } if filters.len() == 256)
        );
        args.extend(["--topic".into(), "topic/256".into()]);
        assert!(parse(args).is_err());

        let mut args = vec!["sub".into(), "--open-lab".into()];
        for index in 0..64 {
            let suffix = index.to_string();
            let prefix = "a".repeat(1024 - suffix.len() - 2);
            args.extend(["--topic".into(), format!("{prefix}{suffix}/+")]);
        }
        assert!(parse(args).is_err());
    }

    #[test]
    fn subscribe_packet_limit_counts_header_and_utf8_bytes_at_exact_boundary() {
        for (tail_bytes, allowed) in [(826, true), (827, false)] {
            let mut filters: Vec<String> = (0..63)
                .map(|index| format!("{index:02}{}", "é".repeat(511)))
                .collect();
            filters.push("a".repeat(tail_bytes));
            assert_eq!(
                super::subscribe_packet_size(&filters),
                Some(65536 + usize::from(!allowed))
            );
            let mut args = vec!["sub".into(), "--open-lab".into()];
            for filter in filters {
                args.extend(["--topic".into(), filter]);
            }
            let result = parse(args);
            assert_eq!(result.is_ok(), allowed);
            if !allowed {
                assert!(result.unwrap_err().contains("64 KiB completos"));
            }
        }
        let mut args = vec![
            "sub".into(),
            "--open-lab".into(),
            "--topic".into(),
            "é".repeat(513),
        ];
        assert!(parse(args.clone()).is_err());
        args[3] = "é".repeat(512);
        assert!(parse(args).is_ok());
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
    fn tls_password_requires_ca_and_rejects_client_identity() {
        let base = [
            "sub",
            "--tls-password",
            "--topic",
            "test/message",
            "--username",
            "student",
        ];
        assert!(parse(base.into_iter().map(str::to_owned)).is_err());
        let mut args = base
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        args.extend(["--ca".into(), "ca.crt".into()]);
        let parsed = parse(args.clone()).unwrap();
        assert_eq!(parsed.transport_mode, TransportMode::TlsPassword);
        assert!(parsed.cert.is_none() && parsed.key.is_none());
        args.extend(["--cert".into(), "client.crt".into()]);
        assert!(parse(args).is_err());
        let mut mixed = base
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        mixed.push("--open-lab".into());
        assert!(parse(mixed).is_err());
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

    #[test]
    fn will_file_is_exclusive_and_validated_before_network() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private-will");
        std::fs::write(&path, [0_u8, 255, 10]).unwrap();
        let base = ["sub", "--open-lab", "--topic", "test/state"];
        let mut args: Vec<String> = base.iter().map(|s| (*s).to_owned()).collect();
        args.extend([
            "--will-topic".into(),
            "will/state".into(),
            "--will-message-file".into(),
            path.to_str().unwrap().into(),
        ]);
        assert_eq!(
            parse(args.clone()).unwrap().will.unwrap().message,
            [0, 255, 10]
        );
        args.extend(["--will-message".into(), "text".into()]);
        assert!(parse(args).is_err());
        let mut args: Vec<String> = base.iter().map(|s| (*s).to_owned()).collect();
        args.extend(["--will-message-file".into(), path.to_str().unwrap().into()]);
        assert!(parse(args).is_err());
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

#[cfg(test)]
mod payload_options_tests {
    use super::*;
    #[test]
    fn sources_are_exclusive_and_lines_require_stream_source() {
        let base = ["pub", "--open-lab", "--topic", "t"];
        for tail in [
            vec!["--stdin"],
            vec!["--message-file", "data"],
            vec!["--stdin", "--line-mode"],
            vec!["--message-file", "data", "--line-mode"],
        ] {
            assert!(parse(base.iter().copied().chain(tail).map(str::to_owned)).is_ok());
        }
        for tail in [
            vec!["--stdin", "--message", "x"],
            vec!["--stdin", "--message-file", "data"],
            vec!["--message", "x", "--message-file", "data"],
            vec!["--message", "x", "--line-mode"],
            vec!["--stdin", "--stdin"],
            vec!["--stdin", "--line-mode", "--line-mode"],
            vec!["--message-file"],
            vec!["--line-mode"],
        ] {
            assert!(parse(base.iter().copied().chain(tail).map(str::to_owned)).is_err());
        }
        for tail in [
            vec!["--stdin"],
            vec!["--message-file", "data"],
            vec!["--line-mode"],
        ] {
            assert!(
                parse(
                    ["sub", "--open-lab", "--topic", "t"]
                        .into_iter()
                        .chain(tail)
                        .map(str::to_owned)
                )
                .is_err()
            );
        }
    }
}
