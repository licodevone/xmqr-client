//! C125: bounded, one-way MQTT 3.1.1 bridge sidecar (experimental).
use std::{
    error::Error,
    fs,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS, Transport};
use rustls::{
    ClientConfig, RootCertStore,
    version::{TLS12, TLS13},
};
use serde::Deserialize;
use tokio::{task::JoinHandle, time::sleep};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
const MAX_CONFIG_BYTES: u64 = 16 * 1024;
const MAX_PEM_BYTES: u64 = 1024 * 1024;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u8,
    source: Endpoint,
    destination: Endpoint,
    route: Route,
    #[serde(default = "default_retries")]
    max_retries: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Endpoint {
    host: String,
    port: u16,
    client_id: String,
    username: String,
    password_file: PathBuf,
    ca: PathBuf,
    cert: PathBuf,
    key: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    source_prefix: String,
    target_prefix: String,
}

fn default_retries() -> u8 {
    5
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn limited(path: &Path, limit: u64) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(invalid("arquivo de configuracao/credencial acima do limite").into());
    }
    Ok(bytes)
}

fn regular(path: &Path) -> AppResult<fs::Metadata> {
    let meta = fs::metadata(path)?;
    if !meta.is_file() {
        return Err(invalid("configuracao/credencial deve ser arquivo regular").into());
    }
    Ok(meta)
}

fn private(path: &Path) -> AppResult<()> {
    let meta = regular(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(
                invalid("configuracao/credencial exige permissao 0600 ou mais restrita").into(),
            );
        }
    }
    Ok(())
}

fn valid_topic(topic: &str) -> bool {
    !topic.is_empty() && topic.len() <= 1024 && !topic.contains(['#', '+', '\0'])
}

fn validate(config: &Config) -> AppResult<()> {
    if config.version != 1 {
        return Err(invalid("versao TOML da bridge desconhecida").into());
    }
    if config.max_retries == 0 || config.max_retries > 20 {
        return Err(invalid("max_retries deve estar entre 1 e 20").into());
    }
    let a = &config.source;
    let b = &config.destination;
    for endpoint in [a, b] {
        if endpoint.host.is_empty()
            || endpoint.host.contains(['/', '\0', ' ', ':'])
            || endpoint.port == 0
            || endpoint.client_id.is_empty()
            || endpoint.client_id.len() > 128
            || endpoint.username.is_empty()
        {
            return Err(invalid("ponta MQTT invalida").into());
        }
        for path in [
            &endpoint.password_file,
            &endpoint.ca,
            &endpoint.cert,
            &endpoint.key,
        ] {
            if !path.is_absolute() {
                return Err(invalid("caminhos da bridge devem ser absolutos").into());
            }
        }
        private(&endpoint.password_file)?;
        private(&endpoint.key)?;
        regular(&endpoint.ca)?;
        regular(&endpoint.cert)?;
    }
    if a.host.eq_ignore_ascii_case(&b.host) && a.port == b.port {
        return Err(invalid("origem e destino devem ser brokers distintos").into());
    }
    if a.client_id == b.client_id {
        return Err(invalid("identidades MQTT das pontas devem ser distintas").into());
    }
    for prefix in [&config.route.source_prefix, &config.route.target_prefix] {
        if !prefix.ends_with('/') || !valid_topic(prefix) || prefix.starts_with('$') {
            return Err(invalid("prefixo de rota invalido").into());
        }
    }
    if config.route.source_prefix == config.route.target_prefix {
        return Err(invalid("prefixos de origem e destino devem diferir").into());
    }
    Ok(())
}

fn mapped_topic(route: &Route, topic: &str) -> Option<String> {
    let suffix = topic.strip_prefix(&route.source_prefix)?;
    let mapped = format!("{}{}", route.target_prefix, suffix);
    valid_topic(&mapped).then_some(mapped)
}

fn password(path: &Path) -> AppResult<String> {
    private(path)?;
    let bytes = limited(path, 4096)?;
    let mut value = String::from_utf8(bytes)?;
    if value.ends_with('\n') {
        value.pop();
    }
    if value.ends_with('\r') {
        value.pop();
    }
    if value.is_empty() || value.contains(['\n', '\r', '\0']) {
        return Err(invalid("senha MQTT invalida").into());
    }
    Ok(value)
}

fn tls(endpoint: &Endpoint) -> AppResult<ClientConfig> {
    let mut roots = RootCertStore::empty();
    let ca = limited(&endpoint.ca, MAX_PEM_BYTES)?;
    let ca_certs = rustls_pemfile::certs(&mut Cursor::new(ca)).collect::<Result<Vec<_>, _>>()?;
    if ca_certs.is_empty() {
        return Err(invalid("CA sem certificado PEM").into());
    }
    for cert in ca_certs {
        roots.add(cert)?;
    }
    let cert = limited(&endpoint.cert, MAX_PEM_BYTES)?;
    let certs = rustls_pemfile::certs(&mut Cursor::new(cert)).collect::<Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        return Err(invalid("certificado cliente ausente").into());
    }
    let key = limited(&endpoint.key, MAX_PEM_BYTES)?;
    let key = rustls_pemfile::private_key(&mut Cursor::new(key))?
        .ok_or_else(|| invalid("chave privada PEM ausente"))?;
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Ok(ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&TLS13, &TLS12])?
        .with_root_certificates(roots)
        .with_client_auth_cert(certs, key)?)
}

fn options(endpoint: &Endpoint) -> AppResult<MqttOptions> {
    let mut options = MqttOptions::new(&endpoint.client_id, &endpoint.host, endpoint.port);
    options.set_transport(Transport::tls_with_config(tls(endpoint)?.into()));
    options.set_credentials(&endpoint.username, password(&endpoint.password_file)?);
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_session(true);
    options.set_max_packet_size(MAX_PAYLOAD_BYTES - 5, MAX_PAYLOAD_BYTES);
    Ok(options)
}

async fn poll_destination(
    mut loopback: rumqttc::EventLoop,
    connected: Arc<AtomicBool>,
    retries: u8,
) -> AppResult<()> {
    let mut failures = 0_u8;
    loop {
        match loopback.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                connected.store(true, Ordering::Release);
                failures = 0;
                eprintln!("Bridge: destino conectado");
            }
            Ok(_) => {}
            Err(_) => {
                connected.store(false, Ordering::Release);
                if failures == 0 {
                    eprintln!("Bridge: destino desconectado");
                }
                failures = failures.saturating_add(1);
                if failures > retries {
                    return Err(invalid("destino indisponivel apos tentativas").into());
                }
                sleep(Duration::from_secs(u64::from(failures.min(5)))).await;
            }
        }
    }
}

async fn run(config: Config) -> AppResult<()> {
    let source = options(&config.source)?;
    let destination = options(&config.destination)?;
    let (source_client, mut source_loop) = AsyncClient::new(source, 8);
    let (destination_client, destination_loop) = AsyncClient::new(destination, 8);
    let destination_connected = Arc::new(AtomicBool::new(false));
    let mut destination_task: JoinHandle<AppResult<()>> = tokio::spawn(poll_destination(
        destination_loop,
        Arc::clone(&destination_connected),
        config.max_retries,
    ));
    let filter = format!("{}#", config.route.source_prefix);
    let mut failures = 0_u8;
    let mut forwarded = 0_u64;
    let mut dropped = 0_u64;
    loop {
        tokio::select! {
            result = &mut destination_task => return result?,
            _ = tokio::signal::ctrl_c() => return Ok(()),
            event = source_loop.poll() => match event {
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    failures = 0;
                    source_client.subscribe(&filter, QoS::AtMostOnce).await?;
                    eprintln!("Bridge: origem conectada");
                }
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    eprintln!("Bridge: origem assinada");
                }
                Ok(Event::Incoming(Incoming::Publish(message))) => {
                    if !destination_connected.load(Ordering::Acquire)
                        || message.payload.len() > MAX_PAYLOAD_BYTES {
                        dropped += 1;
                        if dropped == 1 || dropped.is_multiple_of(100) {
                            eprintln!("Bridge: {dropped} mensagens descartadas (destino indisponivel ou limite)");
                        }
                        continue;
                    }
                    if let Some(topic) = mapped_topic(&config.route, &message.topic) {
                        destination_client.publish(topic, QoS::AtMostOnce, false,
                                                   message.payload).await?;
                        forwarded += 1;
                        if forwarded.is_multiple_of(100) {
                            eprintln!("Bridge: {forwarded} mensagens encaminhadas");
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => {
                    if failures == 0 {
                        eprintln!("Bridge: origem desconectada");
                    }
                    failures = failures.saturating_add(1);
                    if failures > config.max_retries {
                        return Err(invalid("origem indisponivel apos tentativas").into());
                    }
                    sleep(Duration::from_secs(u64::from(failures.min(5)))).await;
                }
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--version" | "-V") {
        println!(
            "mqtt-bridge 0.1.0-dev (xmqr-client {})",
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "Uso: mqtt-bridge --config ARQUIVO_PRIVADO_TOML\nBridge experimental unidirecional TLS/mTLS QoS 0."
        );
        return;
    }
    if args.len() != 2 || args[0] != "--config" {
        eprintln!("Erro: use mqtt-bridge --config ARQUIVO_PRIVADO_TOML");
        std::process::exit(2);
    }
    let result = (|| -> AppResult<Config> {
        let path = Path::new(&args[1]);
        private(path)?;
        let bytes = limited(path, MAX_CONFIG_BYTES)?;
        let config: Config = toml::from_str(std::str::from_utf8(&bytes)?)?;
        validate(&config)?;
        Ok(config)
    })();
    let Ok(config) = result else {
        eprintln!("Erro: configuracao da bridge invalida ou inacessivel");
        std::process::exit(2);
    };
    if run(config).await.is_err() {
        eprintln!("Erro: bridge encerrada por falha de conexao ou configuracao");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, mapped_topic, valid_topic};

    #[test]
    fn prefix_mapping_stays_bounded_and_does_not_match_neighbors() {
        let route = Route {
            source_prefix: "sensor/".into(),
            target_prefix: "mirror/".into(),
        };
        assert_eq!(
            mapped_topic(&route, "sensor/room/temp").as_deref(),
            Some("mirror/room/temp")
        );
        assert_eq!(mapped_topic(&route, "sensors/room/temp"), None);
        assert_eq!(mapped_topic(&route, "sensor/room/#"), None);
        assert!(!valid_topic(&"x".repeat(1025)));
    }
}
