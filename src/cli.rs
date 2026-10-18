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
pub enum ClientIdMode {
    Provided,
    ServerAssigned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReauthMode {
    Off,
    BeforeIo,
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
    pub message_expiry_seconds: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub user_properties: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct Cli {
    pub protocol_version: ProtocolVersion,
    pub auth_method: Option<String>,
    pub reauth_mode: ReauthMode,
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
    pub ws_url: Option<String>,
    pub ca: Option<PathBuf>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub client_id: String,
    pub client_id_mode: ClientIdMode,
    pub username: Option<String>,
    pub password_file: Option<PathBuf>,
    pub qos: QoS,
    pub retain: bool,
    pub clean_session: bool,
    pub clean_start: SessionStart,
    pub session_expiry: u32,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub outbox_dir: Option<PathBuf>,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub subscription_state_dir: Option<PathBuf>,
    pub disconnect_session_expiry: Option<u32>,
    pub message_expiry: Option<u32>,
    pub payload_format: Option<u8>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub publish_user_properties: Vec<(String, String)>,
    pub subscribe_user_properties: Vec<(String, String)>,
    pub unsubscribe_user_properties: Vec<(String, String)>,
    pub receive_maximum: Option<u16>,
    pub maximum_packet_size: Option<u32>,
    pub topic_alias_maximum: Option<u16>,
    pub subscription_options: SubscriptionOptions,
    pub publish_topic_alias: Option<u16>,
    pub transport_mode: TransportMode,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SubscriptionOptions {
    pub retain_handling: u8,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub identifier: Option<usize>,
}

pub const USAGE: &str = "Uso seguro (padrao):\n  mqtt-client pub --topic TOPICO --message TEXTO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n  mqtt-client sub --topic FILTRO --username USUARIO --ca CA.crt --cert client.crt --key client.key [opcoes]\n  mqtt-client pub|sub --profile PERFIL.toml [opcoes]\nTLS com senha, sem certificado de cliente:\n  mqtt-client pub|sub --tls-password --topic TOPICO --username USUARIO --ca CA.crt [opcoes]\n\nLaboratorio aberto:\n  mqtt-client pub|sub --open-lab --topic TOPICO [opcoes]\nLaboratorio com senha em texto claro:\n  mqtt-client pub|sub --plain-auth-lab --topic TOPICO --username USUARIO [opcoes]\n\nWill: --will-topic TOPICO --will-message TEXTO | --will-message-file PATH [--will-qos 0|1|2] [--will-retain true|false].\nOpcoes: --version (sozinho), --allow-remote-wss (somente WSS mTLS remoto), --protocol-version 3.1.1|5 (padrao 3.1.1; MQTT5 inicial pub/sub QoS0/1/2), --message-file PATH | --stdin (pub, alternativos a --message), --line-mode (arquivo/stdin, LF/CRLF por mensagem), --output text|jsonl|hex|raw (hex/raw apenas sub; raw exige --count 1), --reconnect-attempts 0..10 (padrao 0; sub e conexao inicial pub), --keepalive-seconds 1..65535 (padrao 30), --timeout-seconds 1..300 (padrao 8; protocolo, nao espera de mensagens), --total-timeout-seconds 1..86400 (opcional; sessao apos payload inicial/senha, inclusive backoff), --message TEXTO (pub), --repeat 1..1000 --repeat-interval-ms 1..60000 (pub, fonte unica), --count N (sub, mensagens exibidas), --ignore-retained | --retained-only (sub), --exclude-topic FILTRO (sub, repetivel), --host 127.0.0.1, --port 1883, --qos 0|1|2, --retain true|false, --clean-session true|false, --client-id ID.\nSem --password-file, os modos autenticados solicitam a senha sem exibi-la. Nunca informe senha na linha de comando.\nOs modos de laboratorio aceitam somente loopback. --plain-auth-lab envia usuario e senha sem criptografia. Nunca use fora de aula local.\n";

pub const V5_CONNECT_USAGE: &str = "MQTT 5 CONNECT: --server-assigned-client-id (ID vazio, Clean Start verdadeiro; ID atribuido em stderr), --clean-start true|false, --session-expiry-seconds 0..4294967294|never, --disconnect-session-expiry-seconds 0..4294967294|never (pub ou sub --count N; CONNECT 0 permite apenas DISCONNECT 0), --receive-maximum 1..65535 (somente sub), --maximum-packet-size 5..65536 (somente sub), --topic-alias-maximum 0..16 (somente sub), --retain-handling 0|1|2, --no-local, --retain-as-published, --subscription-identifier 1..268435455 (somente sub), --subscribe-user-property CHAVE=VALOR e --unsubscribe-user-property CHAVE=VALOR (sub MQTT 5, repetiveis; 16 pares/2048 bytes por contexto), --topic '$share/grupo/filtro' (somente sub MQTT 5; sem --no-local), --request-problem-info 0|1, --request-response-info 0|1, --connect-user-property CHAVE=VALOR (repetivel; ate 4096 bytes), --will-delay-seconds 0..4294967295 (exige Will). PUBLISH: --message-expiry-seconds 0..4294967295, --payload-format 0|1, --content-type TEXTO (ate 256 bytes; somente pub MQTT 5). Nao inclua segredos em argumentos.\n";

pub const V5_PUBLISH_USAGE: &str = "MQTT 5 PUBLISH adicional: --response-topic TOPICO (sem curingas), --correlation-data-hex HEX (ate 256 bytes), --publish-user-property CHAVE=VALOR (repetivel; ate 16 pares e 2048 bytes), --publish-topic-alias 1..16 (pub repetido; exige broker anunciar suporte).\n";
pub const V5_SUBSCRIPTION_USAGE: &str = "MQTT 5 SUBSCRIBE: --unsubscribe-user-property exige --count N; sem --count, nao ha UNSUBSCRIBE.\n";
pub const AUTH_USAGE: &str = "Autenticacao MQTT 5: --auth-method plain (somente TLS, com --protocol-version 5; senha por arquivo privado ou prompt). --reauth-before-io confirma AUTH PLAIN apos CONNACK e antes de pub/sub; reautenticacao periodica ainda indisponivel.\n";
pub const V5_WILL_USAGE: &str = "MQTT 5 Will: --will-message-expiry-seconds 0..4294967295, --will-payload-format 0|1, --will-content-type TEXTO (ate 256 bytes), --will-response-topic TOPICO (sem curingas), --will-correlation-data-hex HEX (ate 256 bytes), --will-user-property CHAVE=VALOR (repetivel; ate 16 pares e 2048 bytes). Exige --will-topic, mensagem Will e --protocol-version 5.\n";
pub const OUTBOX_USAGE: &str = "Outbox duravel experimental (Linux/WSL): --outbox-dir DIRETORIO existente e privado (0700) exige pub --protocol-version 5 --qos 1|2 --clean-start false --session-expiry-seconds N --client-id ID fixo, uma mensagem. Por --ws-url, somente WSS mTLS e URL exata; WS aberto e manifesto de assinatura por URL sao recusados. Repita o mesmo comando para retomar; nao remova o journal pendente se o broker negar Session Present. O arquivo contem payload em claro sob permissao 0600.\n";
pub const SUBSCRIPTION_STATE_USAGE: &str = "Assinatura persistente experimental (Linux/WSL): --subscription-state-dir DIRETORIO privado (0700) exige sub --protocol-version 5 --clean-start false --session-expiry-seconds N --client-id ID fixo, sem --count. Repita a mesma assinatura para retomar quando o broker confirmar Session Present; sem sessao, o manifesto e preservado e o comando falha. QoS 1/2 recebe ACK apos escrever/flush em stdout; crash pode duplicar a saida, sem garantia de persistencia do consumidor.\n";
pub const WS_USAGE: &str = "WebSocket experimental: --ws-url ws://127.0.0.1:PORTA/CAMINHO exige --open-lab. WSS mTLS local: --ws-url wss://127.0.0.1:PORTA/mqtt exige --ca, --cert, --key e --username, sem --open-lab. IPv6 [::1] tambem aceito. Pub/sub MQTT 3.1.1 ou 5; sem --host/--port ou userinfo. Estado duravel por URL: apenas outbox MQTT 5 QoS 1/2 em WSS mTLS; assinatura persistente por URL indisponivel.\n";

#[allow(clippy::too_many_lines)] // One pass enforces uniqueness and validates CLI modes.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut args = crate::profile::expand(args.into_iter().collect())?.into_iter();
    let command = args.next().ok_or("informe pub ou sub")?;
    if command != "pub" && command != "sub" {
        return Err(format!("comando desconhecido: {command}"));
    }

    let mut output = None;
    let mut protocol_version = None;
    let mut auth_method = None;
    let mut reauth_before_io = false;
    let mut request_problem_info = None;
    let mut request_response_info = None;
    let mut raw_connect_user_properties = Vec::new();
    let mut raw_publish_user_properties = Vec::new();
    let mut raw_subscribe_user_properties = Vec::new();
    let mut raw_unsubscribe_user_properties = Vec::new();
    let mut raw_will_user_properties = Vec::new();
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
    let mut ws_url = None;
    let mut allow_remote_wss = false;
    let mut ca = None;
    let mut cert = None;
    let mut key = None;
    let mut client_id = None;
    let mut server_assigned_client_id = false;
    let mut username = None;
    let mut password_file = None;
    let mut qos = None;
    let mut retain = None;
    let mut clean_session = None;
    let mut clean_start = None;
    let mut session_expiry = None;
    let mut outbox_dir = None;
    let mut subscription_state_dir = None;
    let mut disconnect_session_expiry = None;
    let mut message_expiry = None;
    let mut payload_format = None;
    let mut content_type = None;
    let mut response_topic = None;
    let mut correlation_data_hex = None;
    let mut receive_maximum = None;
    let mut maximum_packet_size = None;
    let mut topic_alias_maximum = None;
    let mut retain_handling = None;
    let mut subscription_identifier = None;
    let mut no_local = false;
    let mut retain_as_published = false;
    let mut publish_topic_alias = None;
    let mut transport_mode = None;
    let mut will_topic = None;
    let mut will_message = None;
    let mut will_message_file = None;
    let mut will_qos = None;
    let mut will_retain = None;
    let mut will_delay = None;
    let mut will_message_expiry = None;
    let mut will_payload_format = None;
    let mut will_content_type = None;
    let mut will_response_topic = None;
    let mut will_correlation_data_hex = None;

    while let Some(flag) = args.next() {
        if flag == "--allow-remote-wss" {
            if std::mem::replace(&mut allow_remote_wss, true) {
                return Err(format!("opcao repetida: {flag}"));
            }
            continue;
        }
        if flag == "--server-assigned-client-id" {
            if std::mem::replace(&mut server_assigned_client_id, true) {
                return Err(format!("opcao repetida: {flag}"));
            }
            continue;
        }
        if flag == "--reauth-before-io" {
            if std::mem::replace(&mut reauth_before_io, true) {
                return Err(format!("opcao repetida: {flag}"));
            }
            continue;
        }
        if flag == "--no-local" || flag == "--retain-as-published" {
            let target = if flag == "--no-local" {
                &mut no_local
            } else {
                &mut retain_as_published
            };
            if std::mem::replace(target, true) {
                return Err(format!("opcao repetida: {flag}"));
            }
            continue;
        }
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
        if flag == "--subscribe-user-property" {
            raw_subscribe_user_properties.push(value);
            continue;
        }
        if flag == "--unsubscribe-user-property" {
            raw_unsubscribe_user_properties.push(value);
            continue;
        }
        if flag == "--will-user-property" {
            raw_will_user_properties.push(value);
            continue;
        }
        let slot = match flag.as_str() {
            "--protocol-version" => &mut protocol_version,
            "--auth-method" => &mut auth_method,
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
            "--retain-handling" => &mut retain_handling,
            "--subscription-identifier" => &mut subscription_identifier,
            "--publish-topic-alias" => &mut publish_topic_alias,
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
            "--will-message-expiry-seconds" => &mut will_message_expiry,
            "--will-payload-format" => &mut will_payload_format,
            "--will-content-type" => &mut will_content_type,
            "--will-response-topic" => &mut will_response_topic,
            "--will-correlation-data-hex" => &mut will_correlation_data_hex,
            "--message-file" => &mut message_file,
            "--message" => &mut message,
            "--count" => &mut count,
            "--host" => &mut host,
            "--port" => &mut port,
            "--ws-url" => &mut ws_url,
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
            "--outbox-dir" => &mut outbox_dir,
            "--subscription-state-dir" => &mut subscription_state_dir,
            "--disconnect-session-expiry-seconds" => &mut disconnect_session_expiry,
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
    if let Mode::Sub { filters, .. } = &mode {
        for filter in filters {
            if let Some(shared) = filter.strip_prefix("$share/") {
                if protocol_version != ProtocolVersion::Mqtt5 {
                    return Err("filtro $share/ exige sub --protocol-version 5".into());
                }
                let Some((group, inner)) = shared.split_once('/') else {
                    return Err("filtro compartilhado exige grupo e filtro".into());
                };
                if group.is_empty()
                    || group.contains(['+', '#'])
                    || inner.is_empty()
                    || !valid_filter(inner)
                {
                    return Err("filtro compartilhado invalido".into());
                }
                if no_local {
                    return Err("--no-local e invalido com filtro compartilhado".into());
                }
            }
        }
    }
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
    if (!raw_subscribe_user_properties.is_empty() || !raw_unsubscribe_user_properties.is_empty())
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("propriedades de assinatura exigem sub --protocol-version 5".into());
    }
    let subscribe_user_properties = parse_subscription_user_properties(
        raw_subscribe_user_properties,
        "--subscribe-user-property",
    )?;
    let unsubscribe_user_properties = parse_subscription_user_properties(
        raw_unsubscribe_user_properties,
        "--unsubscribe-user-property",
    )?;
    if !unsubscribe_user_properties.is_empty() && !matches!(&mode, Mode::Sub { count: Some(_), .. })
    {
        return Err("--unsubscribe-user-property exige sub --count N".into());
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
    if retain_handling.is_some() && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--retain-handling exige sub --protocol-version 5".into());
    }
    let retain_handling = retain_handling
        .map(|raw| {
            raw.parse::<u8>()
                .ok()
                .filter(|value| *value <= 2)
                .ok_or("--retain-handling deve ser 0, 1 ou 2".to_owned())
        })
        .transpose()?
        .unwrap_or(0);
    if (no_local || retain_as_published)
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--no-local/--retain-as-published exigem sub --protocol-version 5".into());
    }
    if subscription_identifier.is_some()
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "sub")
    {
        return Err("--subscription-identifier exige sub --protocol-version 5".into());
    }
    let subscription_identifier = subscription_identifier
        .map(|raw| {
            raw.parse::<usize>()
                .ok()
                .filter(|value| (1..=268_435_455).contains(value))
                .ok_or("--subscription-identifier deve ser 1..268435455".to_owned())
        })
        .transpose()?;
    if publish_topic_alias.is_some()
        && (protocol_version != ProtocolVersion::Mqtt5 || command != "pub" || repeat_count < 2)
    {
        return Err("--publish-topic-alias exige pub --protocol-version 5 --repeat 2..1000".into());
    }
    let publish_topic_alias = publish_topic_alias
        .map(|raw| {
            raw.parse::<u16>()
                .ok()
                .filter(|value| (1..=16).contains(value))
                .ok_or("--publish-topic-alias deve ser 1..16".to_owned())
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
    if outbox_dir.is_some() {
        if !cfg!(target_os = "linux") {
            return Err("--outbox-dir esta disponivel somente no Linux/WSL".into());
        }
        if protocol_version != ProtocolVersion::Mqtt5
            || !matches!(&mode, Mode::Pub { .. })
            || !matches!(qos, QoS::AtLeastOnce | QoS::ExactlyOnce)
            || clean_start != SessionStart::Resume
            || session_expiry == 0
            || server_assigned_client_id
            || client_id.is_none()
            || repeat_count != 1
            || line_mode
            || publish_topic_alias.is_some()
            || will_topic.is_some()
        {
            return Err("--outbox-dir exige pub MQTT 5 QoS 1|2, ID fixo, Clean Start false, Session Expiry positivo e uma mensagem sem Will/alias".into());
        }
    }
    if subscription_state_dir.is_some() {
        if !cfg!(target_os = "linux") {
            return Err("--subscription-state-dir esta disponivel somente no Linux/WSL".into());
        }
        if protocol_version != ProtocolVersion::Mqtt5
            || !matches!(&mode, Mode::Sub { count: None, .. })
            || clean_start != SessionStart::Resume
            || session_expiry == 0
            || server_assigned_client_id
            || client_id.is_none()
            || will_topic.is_some()
        {
            return Err("--subscription-state-dir exige sub MQTT 5 sem --count, ID fixo, Clean Start false e Session Expiry positivo sem Will".into());
        }
    }
    let disconnect_session_expiry = disconnect_session_expiry
        .as_deref()
        .map(|raw| match raw {
            "never" => Ok(u32::MAX),
            value => value
                .parse::<u32>()
                .ok()
                .filter(|seconds| *seconds != u32::MAX)
                .ok_or("--disconnect-session-expiry-seconds deve ser 0..4294967294 ou never"),
        })
        .transpose()?;
    if let Some(seconds) = disconnect_session_expiry {
        if protocol_version != ProtocolVersion::Mqtt5
            || matches!(&mode, Mode::Sub { count: None, .. })
        {
            return Err("--disconnect-session-expiry-seconds exige pub ou sub --count N com --protocol-version 5".into());
        }
        if session_expiry == 0 && seconds != 0 {
            return Err("CONNECT com expiração zero não permite DISCONNECT não zero".into());
        }
        if outbox_dir.is_some() && seconds == 0 {
            return Err("--outbox-dir nao permite encerrar a sessao no DISCONNECT".into());
        }
    }

    let transport_mode = transport_mode.unwrap_or(TransportMode::Secure);
    if let Some(method) = auth_method.as_deref() {
        if method != "plain" {
            return Err("--auth-method aceita somente plain".into());
        }
        if protocol_version != ProtocolVersion::Mqtt5 {
            return Err("--auth-method plain exige --protocol-version 5".into());
        }
        if !matches!(
            transport_mode,
            TransportMode::Secure | TransportMode::TlsPassword
        ) {
            return Err("--auth-method plain exige TLS".into());
        }
    }
    if reauth_before_io && auth_method.as_deref() != Some("plain") {
        return Err("--reauth-before-io exige --auth-method plain".into());
    }
    let lab_mode = matches!(
        transport_mode,
        TransportMode::OpenLab | TransportMode::PlainAuthLab
    );
    if allow_remote_wss && ws_url.is_none() {
        return Err("--allow-remote-wss exige URL WSS remota".into());
    }
    if let Some(value) = ws_url.as_deref() {
        if host.is_some() || port.is_some() {
            return Err("--ws-url nao aceita --host ou --port".into());
        }
        let url = url::Url::parse(value).map_err(|_| "--ws-url invalida")?;
        if subscription_state_dir.is_some() || (outbox_dir.is_some() && url.scheme() != "wss") {
            return Err("--ws-url aceita apenas outbox QoS 1/2 por WSS mTLS; manifesto de assinatura e WS aberto nao aceitam estado persistente".into());
        }
        let valid_mode = matches!(
            (url.scheme(), transport_mode),
            ("ws", TransportMode::OpenLab) | ("wss", TransportMode::Secure)
        );
        let loopback = matches!(
            url.host(),
            Some(url::Host::Ipv4(ip)) if ip == std::net::Ipv4Addr::LOCALHOST
        ) || matches!(
            url.host(),
            Some(url::Host::Ipv6(ip)) if ip == std::net::Ipv6Addr::LOCALHOST
        );
        let local_name = matches!(
            url.host(),
            Some(url::Host::Domain(domain))
                if domain.eq_ignore_ascii_case("localhost") || domain.ends_with(".localhost")
        );
        if !valid_mode
            || local_name
            || (!loopback && !allow_remote_wss)
            || (allow_remote_wss && (loopback || url.scheme() != "wss"))
            || url.port().is_none_or(|port| port == 0)
            || url.path().is_empty()
            || url.path() == "/"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "--ws-url exige ws de laboratorio local ou wss mTLS local/remoto com opt-in, porta e caminho, sem credenciais ou parametros"
                    .into(),
            );
        }
    }
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
    if server_assigned_client_id
        && (client_id.is_some()
            || protocol_version != ProtocolVersion::Mqtt5
            || clean_start != SessionStart::Clean)
    {
        return Err(
            "--server-assigned-client-id exige MQTT 5, Clean Start verdadeiro e sem --client-id"
                .into(),
        );
    }
    let client_id = if server_assigned_client_id {
        String::new()
    } else {
        let client_id =
            client_id.unwrap_or_else(|| format!("mqtt-{command}-{}", std::process::id()));
        let maximum = if protocol_version == ProtocolVersion::Mqtt5 {
            254
        } else {
            23
        };
        if client_id.is_empty()
            || client_id.len() > maximum
            || !client_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(format!(
                "--client-id deve ter 1 a {maximum} caracteres ASCII alfanumericos, '-' ou '_'"
            ));
        }
        client_id
    };

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
    if (will_delay.is_some() || will_message_expiry.is_some())
        && protocol_version != ProtocolVersion::Mqtt5
    {
        return Err("Will Delay/Message Expiry exige --protocol-version 5".into());
    }
    let will_delay = will_delay
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|_| "--will-delay-seconds deve ser 0..4294967295".to_owned())
        })
        .transpose()?;
    let will_message_expiry = will_message_expiry
        .map(|raw| {
            raw.parse::<u32>()
                .map_err(|_| "--will-message-expiry-seconds deve ser 0..4294967295".to_owned())
        })
        .transpose()?;
    if (will_payload_format.is_some()
        || will_content_type.is_some()
        || will_response_topic.is_some()
        || will_correlation_data_hex.is_some()
        || !raw_will_user_properties.is_empty())
        && protocol_version != ProtocolVersion::Mqtt5
    {
        return Err("propriedades Will exigem --protocol-version 5".into());
    }
    let will_payload_format = will_payload_format
        .map(|raw| match raw.as_str() {
            "0" => Ok(0),
            "1" => Ok(1),
            _ => Err("--will-payload-format deve ser 0 ou 1".to_owned()),
        })
        .transpose()?;
    if will_content_type
        .as_ref()
        .is_some_and(|value| value.len() > 256 || !valid_mqtt5_property_text(value))
    {
        return Err(
            "--will-content-type contem texto MQTT 5 invalido ou acima de 256 bytes".into(),
        );
    }
    if will_response_topic.as_ref().is_some_and(|value| {
        value.is_empty()
            || value.len() > MAX_TOPIC_BYTES
            || value.contains(['+', '#'])
            || !valid_mqtt5_property_text(value)
    }) {
        return Err("--will-response-topic exige topico MQTT 5 valido sem curingas".into());
    }
    let will_correlation_data = will_correlation_data_hex
        .map(|raw: String| {
            if raw.len() > 512 || !raw.len().is_multiple_of(2) || !raw.is_ascii() {
                return Err(
                    "--will-correlation-data-hex exige ate 256 bytes em hexadecimal".to_owned(),
                );
            }
            raw.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| {
                    let hex = std::str::from_utf8(pair).expect("ASCII verificado");
                    u8::from_str_radix(hex, 16).map_err(|_| {
                        "--will-correlation-data-hex exige bytes em hexadecimal".to_owned()
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let mut will_user_properties = Vec::new();
    let mut will_property_bytes = 0_usize;
    for raw in raw_will_user_properties {
        let (key, value) = raw
            .split_once('=')
            .ok_or("--will-user-property exige CHAVE=VALOR")?;
        if key.len() > 256
            || value.len() > 256
            || !valid_mqtt5_property_text(key)
            || !valid_mqtt5_property_text(value)
        {
            return Err("--will-user-property contem texto MQTT 5 invalido".into());
        }
        will_property_bytes = will_property_bytes
            .checked_add(5 + key.len() + value.len())
            .ok_or("propriedades Will excedem 2048 bytes")?;
        if will_user_properties.len() >= 16 || will_property_bytes > 2048 {
            return Err("propriedades Will excedem limite de 16 pares ou 2048 bytes".into());
        }
        will_user_properties.push((key.to_owned(), value.to_owned()));
    }
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
            if will_payload_format == Some(1) && std::str::from_utf8(&message).is_err() {
                return Err("--will-payload-format 1 exige payload UTF-8 valido".into());
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
                delay_seconds: will_delay,
                message_expiry_seconds: will_message_expiry,
                payload_format_indicator: will_payload_format,
                content_type: will_content_type,
                response_topic: will_response_topic,
                correlation_data: will_correlation_data,
                user_properties: will_user_properties,
            })
        }
        (None, None, None)
            if will_qos.is_none()
                && will_retain.is_none()
                && will_delay.is_none()
                && will_message_expiry.is_none()
                && will_payload_format.is_none()
                && will_content_type.is_none()
                && will_response_topic.is_none()
                && will_correlation_data.is_none()
                && will_user_properties.is_empty() =>
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
        auth_method,
        reauth_mode: if reauth_before_io {
            ReauthMode::BeforeIo
        } else {
            ReauthMode::Off
        },
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
        ws_url,
        ca,
        cert,
        key,
        client_id,
        client_id_mode: if server_assigned_client_id {
            ClientIdMode::ServerAssigned
        } else {
            ClientIdMode::Provided
        },
        username,
        password_file: password_file.map(PathBuf::from),
        qos,
        retain,
        clean_session,
        clean_start,
        session_expiry,
        outbox_dir: outbox_dir.map(PathBuf::from),
        subscription_state_dir: subscription_state_dir.map(PathBuf::from),
        disconnect_session_expiry,
        message_expiry,
        payload_format,
        content_type,
        response_topic,
        correlation_data,
        publish_user_properties,
        subscribe_user_properties,
        unsubscribe_user_properties,
        receive_maximum,
        maximum_packet_size,
        topic_alias_maximum,
        subscription_options: SubscriptionOptions {
            retain_handling,
            no_local,
            retain_as_published,
            identifier: subscription_identifier,
        },
        publish_topic_alias,
        transport_mode,
    })
}

fn parse_subscription_user_properties(
    raw_properties: Vec<String>,
    flag: &str,
) -> Result<Vec<(String, String)>, String> {
    let mut properties = Vec::new();
    let mut bytes = 0_usize;
    for raw in raw_properties {
        let (key, value) = raw
            .split_once('=')
            .ok_or_else(|| format!("{flag} exige CHAVE=VALOR"))?;
        if key.len() > 256
            || value.len() > 256
            || !valid_mqtt5_property_text(key)
            || !valid_mqtt5_property_text(value)
        {
            return Err(format!("{flag} contem texto MQTT 5 invalido"));
        }
        bytes = bytes
            .checked_add(5 + key.len() + value.len())
            .ok_or_else(|| format!("{flag} excede 2048 bytes"))?;
        if properties.len() >= 16 || bytes > 2048 {
            return Err(format!("{flag} excede 16 pares ou 2048 bytes"));
        }
        properties.push((key.to_owned(), value.to_owned()));
    }
    Ok(properties)
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
    fn mqtt5_will_metadata_is_scoped_bounded_and_ordered() {
        use super::parse;
        let base = [
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--protocol-version",
            "5",
            "--will-topic",
            "last",
            "--will-message",
            "bye",
        ];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let cli = parse(args(&[
            "--will-payload-format",
            "1",
            "--will-content-type",
            "text/plain",
            "--will-response-topic",
            "reply/t",
            "--will-correlation-data-hex",
            "00aF",
            "--will-user-property",
            "a=1",
            "--will-user-property",
            "a=2",
            "--will-delay-seconds",
            "2",
        ]))
        .unwrap();
        let will = cli.will.unwrap();
        assert_eq!(will.payload_format_indicator, Some(1));
        assert_eq!(will.content_type.as_deref(), Some("text/plain"));
        assert_eq!(will.response_topic.as_deref(), Some("reply/t"));
        assert_eq!(will.correlation_data.as_deref(), Some(&[0, 0xaf][..]));
        assert_eq!(
            will.user_properties,
            [("a".into(), "1".into()), ("a".into(), "2".into())]
        );
        assert_eq!(will.delay_seconds, Some(2));
        for extra in [
            vec!["--will-payload-format", "2"],
            vec!["--will-payload-format", "1", "--will-payload-format", "0"],
            vec!["--will-response-topic", "bad/+"],
            vec!["--will-content-type", "bad\0type"],
            vec!["--will-correlation-data-hex", "a"],
            vec!["--will-correlation-data-hex", "xy"],
            vec!["--will-user-property", "missing-equals"],
            vec!["--will-user-property", "x=bad\0value"],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
        let no_will = vec![
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--protocol-version",
            "5",
            "--will-content-type",
            "text/plain",
        ];
        assert!(parse(no_will.into_iter().map(str::to_owned)).is_err());
        let legacy = vec![
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--will-topic",
            "last",
            "--will-message",
            "bye",
            "--will-content-type",
            "text/plain",
        ];
        assert!(parse(legacy.into_iter().map(str::to_owned)).is_err());
        let mut too_many = args(&[]);
        for _ in 0..17 {
            too_many.extend(["--will-user-property".into(), "a=b".into()]);
        }
        assert!(parse(too_many).is_err());
    }

    #[test]
    fn mqtt5_will_message_expiry_requires_complete_will_and_preserves_zero() {
        use super::parse;
        let base = ["pub", "--open-lab", "--topic", "t", "--message", "x"];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        let will = parse(args(&[
            "--protocol-version",
            "5",
            "--will-topic",
            "last",
            "--will-message",
            "bye",
            "--will-message-expiry-seconds",
            "0",
        ]))
        .unwrap()
        .will
        .unwrap();
        assert_eq!(will.message_expiry_seconds, Some(0));
        for extra in [
            vec!["--will-message-expiry-seconds", "1"],
            vec![
                "--protocol-version",
                "5",
                "--will-message-expiry-seconds",
                "1",
            ],
            vec![
                "--protocol-version",
                "5",
                "--will-topic",
                "last",
                "--will-message",
                "bye",
                "--will-message-expiry-seconds",
                "-1",
            ],
            vec![
                "--protocol-version",
                "5",
                "--will-topic",
                "last",
                "--will-message",
                "bye",
                "--will-message-expiry-seconds",
                "4294967296",
            ],
            vec![
                "--protocol-version",
                "5",
                "--will-topic",
                "last",
                "--will-message",
                "bye",
                "--will-message-expiry-seconds",
                "1",
                "--will-message-expiry-seconds",
                "2",
            ],
        ] {
            assert!(parse(args(&extra)).is_err(), "{extra:?}");
        }
    }

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
    fn mqtt5_disconnect_session_expiry_is_scoped_and_bounded() {
        use super::parse;
        let base = [
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--protocol-version",
            "5",
            "--session-expiry-seconds",
            "60",
        ];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(parse(args(&[])).unwrap().disconnect_session_expiry, None);
        for (raw, expected) in [("0", 0), ("30", 30), ("never", u32::MAX)] {
            assert_eq!(
                parse(args(&["--disconnect-session-expiry-seconds", raw]))
                    .unwrap()
                    .disconnect_session_expiry,
                Some(expected)
            );
        }
        for raw in ["-1", "4294967295", "bogus"] {
            assert!(parse(args(&["--disconnect-session-expiry-seconds", raw])).is_err());
        }
        let zero_connect = [
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--protocol-version",
            "5",
        ];
        let zero_args = |expiry| {
            zero_connect
                .iter()
                .chain(["--disconnect-session-expiry-seconds", expiry].iter())
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        assert!(parse(zero_args("0")).is_ok());
        assert!(parse(zero_args("1")).is_err());
        let legacy = [
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "x",
            "--disconnect-session-expiry-seconds",
            "0",
        ];
        assert!(parse(legacy.into_iter().map(str::to_owned)).is_err());
        let sub = [
            "sub",
            "--open-lab",
            "--topic",
            "t",
            "--protocol-version",
            "5",
            "--disconnect-session-expiry-seconds",
            "0",
        ];
        assert!(parse(sub.into_iter().map(str::to_owned)).is_err());
        assert!(parse(sub.into_iter().chain(["--count", "1"]).map(str::to_owned)).is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_qos1_outbox_requires_stable_session_and_one_publication() {
        use super::parse;
        let base = [
            "pub",
            "--open-lab",
            "--topic",
            "test/outbox",
            "--message",
            "data",
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "60",
            "--client-id",
            "durable-device",
            "--outbox-dir",
            "/tmp/xmqr-outbox-test",
        ];
        let arguments = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        };
        assert!(parse(arguments(&[])).unwrap().outbox_dir.is_some());
        let mut qos2 = arguments(&[]);
        let qos_position = qos2.iter().position(|item| item == "--qos").unwrap();
        qos2[qos_position + 1] = "2".to_owned();
        assert!(parse(qos2).unwrap().outbox_dir.is_some());
        for extra in [
            &["--repeat", "2", "--repeat-interval-ms", "1"][..],
            &["--will-topic", "last", "--will-message", "bye"],
            &["--disconnect-session-expiry-seconds", "0"],
        ] {
            assert!(parse(arguments(extra)).is_err(), "{extra:?}");
        }
        for invalid in [
            ["--clean-start", "true"],
            ["--session-expiry-seconds", "0"],
            ["--qos", "0"],
            ["--protocol-version", "3.1.1"],
        ] {
            let mut args = arguments(&[]);
            let position = args.iter().position(|item| item == invalid[0]).unwrap();
            args[position + 1] = invalid[1].to_owned();
            assert!(parse(args).is_err(), "{invalid:?}");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_subscription_requires_stable_uncounted_mqtt5_session() {
        use super::parse;
        let base = [
            "sub",
            "--open-lab",
            "--topic",
            "test/#",
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "60",
            "--client-id",
            "durable-sub",
            "--subscription-state-dir",
            "/tmp/xmqr-sub-test",
        ];
        let args = |extra: &[&str]| {
            base.iter()
                .chain(extra)
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>()
        };
        assert!(parse(args(&[])).unwrap().subscription_state_dir.is_some());
        assert!(parse(args(&["--count", "1"])).is_err());
        assert!(parse(args(&["--will-topic", "last", "--will-message", "bye"])).is_err());
        for (flag, bad) in [
            ("--protocol-version", "3.1.1"),
            ("--clean-start", "true"),
            ("--session-expiry-seconds", "0"),
        ] {
            let mut input = args(&[]);
            let index = input.iter().position(|value| value == flag).unwrap();
            input[index + 1] = bad.into();
            assert!(parse(input).is_err(), "{flag}");
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
    fn assigned_client_id_allows_durable_clean_start_mqtt5_without_manual_id() {
        let mut accepted = base("sub");
        accepted.extend([
            "--protocol-version".into(),
            "5".into(),
            "--server-assigned-client-id".into(),
        ]);
        let cli = parse(accepted.clone()).unwrap();
        assert_eq!(cli.client_id_mode, super::ClientIdMode::ServerAssigned);
        assert_eq!(cli.client_id, "");
        for (expiry, expected) in [("0", 0), ("60", 60), ("never", u32::MAX)] {
            let mut args = accepted.clone();
            args.extend(
                ["--session-expiry-seconds", expiry]
                    .into_iter()
                    .map(str::to_owned),
            );
            assert_eq!(parse(args).unwrap().session_expiry, expected);
        }
        for extra in [
            vec!["--client-id", "manual"],
            vec!["--clean-start", "false"],
            vec!["--server-assigned-client-id"],
        ] {
            let mut args = accepted.clone();
            args.extend(extra.into_iter().map(str::to_owned));
            assert!(parse(args).is_err());
        }
        let mut legacy = base("sub");
        legacy.push("--server-assigned-client-id".into());
        assert!(parse(legacy).is_err());
    }

    #[test]
    fn mqtt5_reuses_assigned_length_ids_but_legacy_keeps_23_byte_limit() {
        for (protocol, length, accepted) in [
            ("3.1.1", 23, true),
            ("3.1.1", 24, false),
            ("5", 23, true),
            ("5", 24, true),
            ("5", 37, true),
            ("5", 254, true),
            ("5", 255, false),
            ("5", 0, false),
        ] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                protocol.into(),
                "--client-id".into(),
                "x".repeat(length),
            ]);
            assert_eq!(parse(args).is_ok(), accepted, "{protocol} length {length}");
        }
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
    fn publish_topic_alias_requires_mqtt5_repeated_pub_and_valid_range() {
        for alias in ["1", "16"] {
            let mut args = base("pub");
            args.extend([
                "--message".into(),
                "x".into(),
                "--protocol-version".into(),
                "5".into(),
                "--repeat".into(),
                "2".into(),
                "--repeat-interval-ms".into(),
                "1".into(),
                "--publish-topic-alias".into(),
                alias.into(),
            ]);
            assert_eq!(parse(args).unwrap().publish_topic_alias, alias.parse().ok());
        }
        for alias in ["0", "17", "-1", "x"] {
            let mut args = base("pub");
            args.extend([
                "--message".into(),
                "x".into(),
                "--protocol-version".into(),
                "5".into(),
                "--repeat".into(),
                "2".into(),
                "--repeat-interval-ms".into(),
                "1".into(),
                "--publish-topic-alias".into(),
                alias.into(),
            ]);
            assert!(parse(args).is_err());
        }
        for (mode, mqtt5, repeat) in [
            ("pub", false, true),
            ("pub", true, false),
            ("sub", true, false),
        ] {
            let mut args = base(mode);
            if mode == "pub" {
                args.extend(["--message".into(), "x".into()]);
            }
            if mqtt5 {
                args.extend(["--protocol-version".into(), "5".into()]);
            }
            if repeat {
                args.extend([
                    "--repeat".into(),
                    "2".into(),
                    "--repeat-interval-ms".into(),
                    "1".into(),
                ]);
            }
            args.extend(["--publish-topic-alias".into(), "1".into()]);
            assert!(parse(args).is_err());
        }
    }

    #[test]
    fn retain_handling_is_sub_only_mqtt5_and_bounded() {
        for (raw, expected) in [("0", 0), ("1", 1), ("2", 2)] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                "5".into(),
                "--retain-handling".into(),
                raw.into(),
            ]);
            assert_eq!(
                parse(args).unwrap().subscription_options.retain_handling,
                expected
            );
        }
        for raw in ["3", "-1", "x", "256"] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                "5".into(),
                "--retain-handling".into(),
                raw.into(),
            ]);
            assert!(parse(args).is_err());
        }
        let mut mqtt311 = base("sub");
        mqtt311.extend(["--retain-handling".into(), "1".into()]);
        assert!(parse(mqtt311).is_err());
        let mut publisher = base("pub");
        publisher.extend([
            "--message".into(),
            "x".into(),
            "--protocol-version".into(),
            "5".into(),
            "--retain-handling".into(),
            "1".into(),
        ]);
        assert!(parse(publisher).is_err());
    }

    #[test]
    fn no_local_and_retain_as_published_require_mqtt5_sub() {
        let mut allowed = base("sub");
        allowed.extend([
            "--protocol-version".into(),
            "5".into(),
            "--no-local".into(),
            "--retain-as-published".into(),
            "--retain-handling".into(),
            "2".into(),
        ]);
        let parsed = parse(allowed).unwrap();
        assert!(
            parsed.subscription_options.no_local && parsed.subscription_options.retain_as_published
        );
        assert_eq!(parsed.subscription_options.retain_handling, 2);
        for flag in ["--no-local", "--retain-as-published"] {
            let mut mqtt311 = base("sub");
            mqtt311.push(flag.into());
            assert!(parse(mqtt311).is_err());
            let mut publisher = base("pub");
            publisher.extend([
                "--message".into(),
                "x".into(),
                "--protocol-version".into(),
                "5".into(),
                flag.into(),
            ]);
            assert!(parse(publisher).is_err());
            let mut duplicate = base("sub");
            duplicate.extend([
                "--protocol-version".into(),
                "5".into(),
                flag.into(),
                flag.into(),
            ]);
            assert!(parse(duplicate).is_err());
        }
    }

    #[test]
    fn subscription_user_properties_require_mqtt5_sub_and_are_bounded() {
        let mut args = base("sub");
        args.extend([
            "--protocol-version".into(),
            "5".into(),
            "--count".into(),
            "1".into(),
            "--subscription-identifier".into(),
            "3".into(),
            "--subscribe-user-property".into(),
            "trace=one".into(),
            "--subscribe-user-property".into(),
            "trace=two".into(),
            "--unsubscribe-user-property".into(),
            "done=yes".into(),
        ]);
        let parsed = parse(args).unwrap();
        assert_eq!(
            parsed.subscribe_user_properties,
            [
                ("trace".into(), "one".into()),
                ("trace".into(), "two".into())
            ]
        );
        assert_eq!(
            parsed.unsubscribe_user_properties,
            [("done".into(), "yes".into())]
        );
        assert_eq!(parsed.subscription_options.identifier, Some(3));
        let mut no_count = base("sub");
        no_count.extend([
            "--protocol-version".into(),
            "5".into(),
            "--unsubscribe-user-property".into(),
            "done=yes".into(),
        ]);
        assert!(parse(no_count).is_err());
        for flag in ["--subscribe-user-property", "--unsubscribe-user-property"] {
            let mut mqtt311 = base("sub");
            mqtt311.extend([flag.into(), "k=v".into()]);
            assert!(parse(mqtt311).is_err());
            let mut publisher = base("pub");
            publisher.extend([
                "--protocol-version".into(),
                "5".into(),
                "--message".into(),
                "x".into(),
                flag.into(),
                "k=v".into(),
            ]);
            assert!(parse(publisher).is_err());
            for bad in ["missing", "bad\0key=v", "k=\u{0001}"] {
                let mut args = base("sub");
                args.extend([
                    "--protocol-version".into(),
                    "5".into(),
                    flag.into(),
                    bad.into(),
                ]);
                assert!(parse(args).is_err());
            }
            let mut long = base("sub");
            long.extend([
                "--protocol-version".into(),
                "5".into(),
                flag.into(),
                format!("k={}", "v".repeat(257)),
            ]);
            assert!(parse(long).is_err());
            let mut many = base("sub");
            many.extend(["--protocol-version".into(), "5".into()]);
            for _ in 0..17 {
                many.extend([flag.into(), "k=v".into()]);
            }
            assert!(parse(many).is_err());
            let mut large = base("sub");
            large.extend(["--protocol-version".into(), "5".into()]);
            for _ in 0..8 {
                large.extend([
                    flag.into(),
                    format!("{}={}", "k".repeat(128), "v".repeat(128)),
                ]);
            }
            assert!(parse(large).is_err());
        }
    }

    #[test]
    fn subscription_identifier_requires_mqtt5_sub_and_valid_range() {
        for value in ["1", "268435455"] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                "5".into(),
                "--subscription-identifier".into(),
                value.into(),
            ]);
            assert_eq!(
                parse(args).unwrap().subscription_options.identifier,
                Some(value.parse().unwrap())
            );
        }
        for value in ["0", "268435456", "-1", "x"] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                "5".into(),
                "--subscription-identifier".into(),
                value.into(),
            ]);
            assert!(parse(args).is_err());
        }
        let mut mqtt311 = base("sub");
        mqtt311.extend(["--subscription-identifier".into(), "1".into()]);
        assert!(parse(mqtt311).is_err());
        let mut publisher = base("pub");
        publisher.extend([
            "--message".into(),
            "x".into(),
            "--protocol-version".into(),
            "5".into(),
            "--subscription-identifier".into(),
            "1".into(),
        ]);
        assert!(parse(publisher).is_err());
        let mut duplicate = base("sub");
        duplicate.extend([
            "--protocol-version".into(),
            "5".into(),
            "--subscription-identifier".into(),
            "1".into(),
            "--subscription-identifier".into(),
            "2".into(),
        ]);
        assert!(parse(duplicate).is_err());
    }

    #[test]
    fn shared_subscription_filter_requires_mqtt5_valid_group_and_no_local_disabled() {
        let shared = "$share/workers/sensor/+";
        let mut valid = base("sub");
        valid.extend([
            "--protocol-version".into(),
            "5".into(),
            "--topic".into(),
            shared.into(),
        ]);
        let parsed = parse(valid.clone()).unwrap();
        assert!(
            matches!(parsed.mode, super::Mode::Sub { filters, .. } if filters.contains(&shared.to_owned()))
        );
        for filter in [
            "$share//sensor",
            "$share/+/sensor",
            "$share/#/sensor",
            "$share/workers/",
            "$share/workers/sensor/#/bad",
        ] {
            let mut args = base("sub");
            args.extend([
                "--protocol-version".into(),
                "5".into(),
                "--topic".into(),
                filter.into(),
            ]);
            assert!(parse(args).is_err(), "accepted {filter}");
        }
        let mut mqtt311 = base("sub");
        mqtt311.extend(["--topic".into(), shared.into()]);
        assert!(parse(mqtt311).is_err());
        valid.push("--no-local".into());
        assert!(parse(valid).is_err());
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
    fn websocket_url_is_explicit_loopback_lab_only() {
        let base = [
            "pub",
            "--open-lab",
            "--topic",
            "test/value",
            "--message",
            "ok",
        ];
        for url in ["ws://127.0.0.1:8083/mqtt", "ws://[::1]:8083/mqtt"] {
            let parsed =
                parse(base.into_iter().chain(["--ws-url", url]).map(str::to_owned)).unwrap();
            assert_eq!(parsed.ws_url.as_deref(), Some(url));
        }
        for url in [
            "ws://192.168.0.1:8083/mqtt",
            "ws://localhost:8083/mqtt",
            "wss://127.0.0.1:8083/mqtt",
            "http://127.0.0.1:8083/mqtt",
            "ws://127.0.0.1/mqtt",
            "ws://127.0.0.1:0/mqtt",
            "ws://127.0.0.1:8083/",
            "ws://user:pass@127.0.0.1:8083/mqtt",
            "ws://127.0.0.1:8083/mqtt?token=x",
            "ws://127.0.0.1:8083/mqtt#fragment",
        ] {
            assert!(
                parse(base.into_iter().chain(["--ws-url", url]).map(str::to_owned)).is_err(),
                "unexpected URL: {url}"
            );
        }
        for extra in [
            vec!["--host", "127.0.0.1"],
            vec!["--port", "1883"],
            vec!["--outbox-dir", "/tmp/outbox"],
            vec!["--subscription-state-dir", "/tmp/sub"],
        ] {
            assert!(
                parse(
                    base.into_iter()
                        .chain(["--ws-url", "ws://127.0.0.1:8083/mqtt"])
                        .chain(extra)
                        .map(str::to_owned)
                )
                .is_err()
            );
        }
        assert!(
            parse(
                [
                    "pub",
                    "--topic",
                    "test/value",
                    "--message",
                    "ok",
                    "--ws-url",
                    "ws://127.0.0.1:8083/mqtt"
                ]
                .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn wss_url_requires_loopback_mtls_without_destination_or_durable_state() {
        let base = [
            "pub",
            "--topic",
            "test/value",
            "--message",
            "ok",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--cert",
            "client.crt",
            "--key",
            "client.key",
            "--password-file",
            "/tmp/password",
        ];
        for url in ["wss://127.0.0.1:8084/mqtt", "wss://[::1]:8084/mqtt"] {
            let parsed =
                parse(base.into_iter().chain(["--ws-url", url]).map(str::to_owned)).unwrap();
            assert_eq!(parsed.ws_url.as_deref(), Some(url));
            assert_eq!(parsed.transport_mode, TransportMode::Secure);
        }
        for url in [
            "ws://127.0.0.1:8084/mqtt",
            "wss://localhost:8084/mqtt",
            "wss://192.0.2.1:8084/mqtt",
            "wss://127.0.0.1/mqtt",
            "wss://127.0.0.1:0/mqtt",
            "wss://127.0.0.1:8084/",
            "wss://user:pass@127.0.0.1:8084/mqtt",
            "wss://127.0.0.1:8084/mqtt?token=x",
            "wss://127.0.0.1:8084/mqtt#fragment",
        ] {
            assert!(
                parse(base.into_iter().chain(["--ws-url", url]).map(str::to_owned)).is_err(),
                "unexpected WSS URL: {url}"
            );
        }
        for extra in [
            vec!["--host", "127.0.0.1"],
            vec!["--port", "8883"],
            vec!["--outbox-dir", "/tmp/outbox"],
            vec!["--subscription-state-dir", "/tmp/sub"],
            vec!["--tls-password"],
        ] {
            assert!(
                parse(
                    base.into_iter()
                        .chain(["--ws-url", "wss://127.0.0.1:8084/mqtt"])
                        .chain(extra)
                        .map(str::to_owned)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn wss_qos1_qos2_outbox_requires_secure_url_and_persistent_session() {
        let base = [
            "pub",
            "--topic",
            "test/value",
            "--message",
            "ok",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--cert",
            "client.crt",
            "--key",
            "client.key",
            "--password-file",
            "/tmp/password",
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--client-id",
            "fixed-id",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "60",
            "--outbox-dir",
            "/tmp/outbox",
        ];
        let local = base
            .into_iter()
            .chain(["--ws-url", "wss://127.0.0.1:8084/mqtt"]);
        assert!(
            parse(local.map(str::to_owned))
                .unwrap()
                .outbox_dir
                .is_some()
        );
        let remote = base.into_iter().chain([
            "--allow-remote-wss",
            "--ws-url",
            "wss://broker.example:8084/mqtt",
        ]);
        assert!(
            parse(remote.map(str::to_owned))
                .unwrap()
                .outbox_dir
                .is_some()
        );
        for tail in [
            vec!["--ws-url", "ws://127.0.0.1:8084/mqtt"],
            vec!["--ws-url", "wss://broker.example:8084/mqtt"],
        ] {
            assert!(parse(base.into_iter().chain(tail).map(str::to_owned)).is_err());
        }
        let qos2 = base.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let qos_index = qos2.iter().position(|value| value == "--qos").unwrap() + 1;
        let mut qos2 = qos2;
        qos2[qos_index] = "2".to_owned();
        qos2.extend(["--ws-url", "wss://127.0.0.1:8084/mqtt"].map(str::to_owned));
        assert!(parse(qos2).unwrap().outbox_dir.is_some());
        let mut remote_qos2 = base.into_iter().map(str::to_owned).collect::<Vec<_>>();
        remote_qos2[qos_index] = "2".to_owned();
        remote_qos2.extend(
            [
                "--allow-remote-wss",
                "--ws-url",
                "wss://broker.example:8084/mqtt",
            ]
            .map(str::to_owned),
        );
        assert!(parse(remote_qos2).unwrap().outbox_dir.is_some());
    }

    #[test]
    fn remote_wss_requires_explicit_opt_in_and_secure_url() {
        let base = [
            "pub",
            "--topic",
            "test/value",
            "--message",
            "ok",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--cert",
            "client.crt",
            "--key",
            "client.key",
            "--password-file",
            "/tmp/password",
        ];
        for url in [
            "wss://broker.example:8084/mqtt",
            "wss://192.0.2.1:8084/mqtt",
        ] {
            assert!(parse(base.into_iter().chain(["--ws-url", url]).map(str::to_owned)).is_err());
            let parsed = parse(
                base.into_iter()
                    .chain(["--allow-remote-wss", "--ws-url", url])
                    .map(str::to_owned),
            )
            .unwrap();
            assert_eq!(parsed.ws_url.as_deref(), Some(url));
        }
        for tail in [
            vec!["--allow-remote-wss"],
            vec![
                "--allow-remote-wss",
                "--ws-url",
                "wss://127.0.0.1:8084/mqtt",
            ],
            vec![
                "--allow-remote-wss",
                "--ws-url",
                "wss://localhost:8084/mqtt",
            ],
            vec![
                "--allow-remote-wss",
                "--ws-url",
                "ws://broker.example:8084/mqtt",
            ],
            vec![
                "--allow-remote-wss",
                "--ws-url",
                "wss://user:secret@broker.example:8084/mqtt",
            ],
            vec![
                "--allow-remote-wss",
                "--ws-url",
                "wss://broker.example:8084/mqtt?x=1",
            ],
            vec![
                "--allow-remote-wss",
                "--allow-remote-wss",
                "--ws-url",
                "wss://broker.example:8084/mqtt",
            ],
        ] {
            let error = parse(base.into_iter().chain(tail).map(str::to_owned)).unwrap_err();
            assert!(!error.contains("secret"));
        }
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
    fn plain_method_requires_mqtt5_and_tls() {
        let mut secure = base("sub");
        secure.extend([
            "--protocol-version".into(),
            "5".into(),
            "--auth-method".into(),
            "plain".into(),
        ]);
        assert_eq!(
            parse(secure.clone()).unwrap().auth_method.as_deref(),
            Some("plain")
        );
        let mut invalid = base("sub");
        invalid.extend(["--auth-method".into(), "plain".into()]);
        assert!(parse(invalid).is_err());
        let mut invalid = secure.clone();
        invalid.pop();
        invalid.push("PLAIN".into());
        assert!(parse(invalid).is_err());
        let lab = [
            "sub",
            "--open-lab",
            "--topic",
            "test/message",
            "--protocol-version",
            "5",
            "--auth-method",
            "plain",
        ];
        assert!(parse(lab.into_iter().map(str::to_owned)).is_err());
    }

    #[test]
    fn reauth_requires_explicit_plain_method() {
        let mut args = base("sub");
        args.push("--reauth-before-io".into());
        assert!(parse(args.clone()).is_err());
        args.extend([
            "--protocol-version".into(),
            "5".into(),
            "--auth-method".into(),
            "plain".into(),
        ]);
        assert_eq!(
            parse(args.clone()).unwrap().reauth_mode,
            super::ReauthMode::BeforeIo
        );
        args.push("--reauth-before-io".into());
        assert!(parse(args).is_err());
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
