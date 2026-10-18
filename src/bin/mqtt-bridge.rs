//! C125-C137: bounded, one-way MQTT bridge sidecar (experimental).
#[path = "../bridge_journal.rs"]
mod bridge_journal;
use std::{
    error::Error,
    fs,
    io::{self, Cursor, Read},
    net::ToSocketAddrs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bridge_journal::Journal;
#[cfg(debug_assertions)]
use rumqttc::Outgoing;
use rumqttc::v5::{
    self,
    mqttbytes::{
        QoS as QoS5,
        v5::{ConnectReturnCode, Filter, Publish as Publish5, SubscribeReasonCode},
    },
};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Publish, QoS, Transport};
use rustls::{
    ClientConfig, RootCertStore,
    version::{TLS12, TLS13},
};
use serde::Deserialize;
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, timeout},
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
const MAX_CONFIG_BYTES: u64 = 16 * 1024;
const MAX_PEM_BYTES: u64 = 1024 * 1024;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u8,
    #[serde(default)]
    source_protocol: SourceProtocol,
    source: Endpoint,
    destination: Endpoint,
    route: Route,
    topology_file: Option<PathBuf>,
    link_id: Option<String>,
    journal_dir: Option<PathBuf>,
    #[serde(default = "default_retries")]
    max_retries: u8,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SourceProtocol {
    #[default]
    Mqtt311,
    Mqtt5,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Endpoint {
    broker_id: Option<String>,
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
    #[serde(default)]
    qos: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Topology {
    version: u8,
    brokers: Vec<BrokerIdentity>,
    links: Vec<Link>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrokerIdentity {
    id: String,
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    id: String,
    source_broker: String,
    destination_broker: String,
    source_prefix: String,
    target_prefix: String,
    qos: u8,
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
    #[cfg(not(unix))]
    let _ = &meta;
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

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_prefix(prefix: &str) -> bool {
    prefix.ends_with('/') && valid_topic(prefix) && !prefix.starts_with('$')
}

fn prefixes_overlap(a: &str, b: &str) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

fn same_endpoint(a_host: &str, a_port: u16, b_host: &str, b_port: u16) -> AppResult<bool> {
    if a_port != b_port {
        return Ok(false);
    }
    if a_host.eq_ignore_ascii_case(b_host) {
        return Ok(true);
    }
    let a_addresses = (a_host, a_port).to_socket_addrs()?.collect::<Vec<_>>();
    let b_addresses = (b_host, b_port).to_socket_addrs()?.collect::<Vec<_>>();
    if a_addresses.is_empty() || b_addresses.is_empty() {
        return Err(invalid("endpoint sem endereco resolvido").into());
    }
    Ok(a_addresses
        .iter()
        .any(|address| b_addresses.contains(address)))
}

fn visit_link(index: usize, links: &[Link], state: &mut [u8]) -> bool {
    if state[index] == 1 {
        return true;
    }
    if state[index] == 2 {
        return false;
    }
    state[index] = 1;
    for (next, candidate) in links.iter().enumerate() {
        if links[index].destination_broker == candidate.source_broker
            && prefixes_overlap(&links[index].target_prefix, &candidate.source_prefix)
            && visit_link(next, links, state)
        {
            return true;
        }
    }
    state[index] = 2;
    false
}

fn has_cycle(links: &[Link]) -> bool {
    let mut state = vec![0_u8; links.len()];
    (0..links.len()).any(|index| visit_link(index, links, &mut state))
}

fn check_topology(config: &Config) -> AppResult<()> {
    let path = config
        .topology_file
        .as_ref()
        .ok_or_else(|| invalid("QoS 1/2 exige topology_file"))?;
    if !path.is_absolute() {
        return Err(invalid("topology_file deve ser absoluto").into());
    }
    private(path)?;
    let contents = limited(path, MAX_CONFIG_BYTES)?;
    let topology: Topology = toml::from_str(std::str::from_utf8(&contents)?)?;
    if topology.version != 1
        || topology.brokers.len() < 2
        || topology.brokers.len() > 64
        || topology.links.is_empty()
        || topology.links.len() > 128
    {
        return Err(invalid("topologia invalida ou acima do limite").into());
    }
    for (index, broker) in topology.brokers.iter().enumerate() {
        if !valid_id(&broker.id)
            || broker.host.is_empty()
            || broker.host.contains(['/', '\0', ' ', ':'])
            || broker.port == 0
            || topology.brokers[..index]
                .iter()
                .any(|previous| previous.id == broker.id)
        {
            return Err(invalid("identidade ou endereco duplicado na topologia").into());
        }
        for previous in &topology.brokers[..index] {
            if same_endpoint(&previous.host, previous.port, &broker.host, broker.port)? {
                return Err(invalid("endereco duplicado na topologia").into());
            }
        }
    }
    let matches_endpoint = |endpoint: &Endpoint| {
        topology.brokers.iter().any(|broker| {
            endpoint.broker_id.as_deref() == Some(broker.id.as_str())
                && endpoint.host.eq_ignore_ascii_case(&broker.host)
                && endpoint.port == broker.port
        })
    };
    if !matches_endpoint(&config.source) || !matches_endpoint(&config.destination) {
        return Err(invalid("ponta nao corresponde a topologia").into());
    }
    let mut own_links = 0;
    for (index, link) in topology.links.iter().enumerate() {
        if !valid_id(&link.id)
            || !valid_prefix(&link.source_prefix)
            || !valid_prefix(&link.target_prefix)
            || link.qos > 2
            || link.source_broker == link.destination_broker
            || !topology
                .brokers
                .iter()
                .any(|broker| broker.id == link.source_broker)
            || !topology
                .brokers
                .iter()
                .any(|broker| broker.id == link.destination_broker)
            || topology.links[..index]
                .iter()
                .any(|previous| previous.id == link.id)
        {
            return Err(invalid("link invalido na topologia").into());
        }
        if config.link_id.as_deref() == Some(link.id.as_str()) {
            own_links += 1;
            if config.source.broker_id.as_deref() != Some(link.source_broker.as_str())
                || config.destination.broker_id.as_deref() != Some(link.destination_broker.as_str())
                || config.route.source_prefix != link.source_prefix
                || config.route.target_prefix != link.target_prefix
                || config.route.qos != link.qos
            {
                return Err(invalid("rota local diverge da topologia").into());
            }
        }
    }
    if own_links != 1 {
        return Err(invalid("link_id ausente da topologia").into());
    }
    if has_cycle(&topology.links) {
        return Err(invalid("ciclo de bridge na topologia").into());
    }
    Ok(())
}

fn validate(config: &Config) -> AppResult<()> {
    if config.version != 1 {
        return Err(invalid("versao TOML da bridge desconhecida").into());
    }
    if config.max_retries == 0 || config.max_retries > 20 {
        return Err(invalid("max_retries deve estar entre 1 e 20").into());
    }
    if config.source_protocol == SourceProtocol::Mqtt5 && config.route.qos == 2 {
        return Err(invalid("origem MQTT 5 nao suporta route.qos=2 nesta etapa").into());
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
    if same_endpoint(&a.host, a.port, &b.host, b.port)? {
        return Err(invalid("origem e destino devem ser brokers distintos").into());
    }
    if a.client_id == b.client_id {
        return Err(invalid("identidades MQTT das pontas devem ser distintas").into());
    }
    for prefix in [&config.route.source_prefix, &config.route.target_prefix] {
        if !valid_prefix(prefix) {
            return Err(invalid("prefixo de rota invalido").into());
        }
    }
    if config.route.source_prefix == config.route.target_prefix {
        return Err(invalid("prefixos de origem e destino devem diferir").into());
    }
    if config.route.qos > 2 {
        return Err(invalid("bridge aceita somente QoS 0, 1 ou 2").into());
    }
    if config.route.qos == 2 {
        let directory = config
            .journal_dir
            .as_ref()
            .ok_or_else(|| invalid("QoS 2 exige journal_dir privado"))?;
        if !directory.is_absolute() {
            return Err(invalid("journal_dir deve ser absoluto").into());
        }
    } else if config.journal_dir.is_some() {
        return Err(invalid("journal_dir so se aplica a QoS 2").into());
    }
    if config.route.qos > 0 || config.topology_file.is_some() || config.link_id.is_some() {
        check_topology(config)?;
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

fn options(
    endpoint: &Endpoint,
    persistent_session: bool,
    manual_acks: bool,
) -> AppResult<MqttOptions> {
    let mut options = MqttOptions::new(&endpoint.client_id, &endpoint.host, endpoint.port);
    options.set_transport(Transport::tls_with_config(tls(endpoint)?.into()));
    options.set_credentials(&endpoint.username, password(&endpoint.password_file)?);
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_session(!persistent_session);
    options.set_manual_acks(manual_acks);
    options.set_max_packet_size(MAX_PAYLOAD_BYTES - 5, MAX_PAYLOAD_BYTES);
    Ok(options)
}

fn options_v5(endpoint: &Endpoint, qos: u8) -> AppResult<v5::MqttOptions> {
    let mut options = v5::MqttOptions::new(&endpoint.client_id, &endpoint.host, endpoint.port);
    options.set_transport(Transport::tls_with_config(tls(endpoint)?.into()));
    options.set_credentials(&endpoint.username, password(&endpoint.password_file)?);
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_start(qos == 0);
    if qos == 1 {
        options.set_session_expiry_interval(Some(86_400));
        options.set_manual_acks(true);
    }
    options.set_max_packet_size(Some(u32::try_from(MAX_PAYLOAD_BYTES)?));
    Ok(options)
}

async fn forward_mqtt5_qos1(
    config: &Config,
    message: &Publish5,
    source_client: &v5::AsyncClient,
    destination_client: &AsyncClient,
    destination_connected: &AtomicBool,
    ack_rx: &mut mpsc::Receiver<()>,
) -> AppResult<()> {
    if message.qos != QoS5::AtLeastOnce || message.payload.len() > MAX_PAYLOAD_BYTES {
        return Err(invalid("mensagem MQTT 5 QoS invalida ou acima do limite").into());
    }
    let topic = std::str::from_utf8(&message.topic)
        .ok()
        .and_then(|topic| mapped_topic(&config.route, topic))
        .ok_or_else(|| invalid("topico MQTT 5 de origem nao mapeavel"))?;
    timeout(Duration::from_secs(20), async {
        while !destination_connected.load(Ordering::Acquire) {
            sleep(Duration::from_millis(50)).await;
        }
        destination_client
            .publish(
                topic,
                QoS::AtLeastOnce,
                message.retain,
                message.payload.clone(),
            )
            .await?;
        ack_rx
            .recv()
            .await
            .ok_or_else(|| invalid("destino sem PUBACK"))?;
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await
    .map_err(|_| invalid("destino sem PUBACK no prazo"))??;
    #[cfg(debug_assertions)]
    if std::env::var_os("XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_ACK").is_some()
        && message.topic.ends_with(b"/crash")
    {
        eprintln!("Bridge: checkpoint apos PUBACK do destino");
        sleep(Duration::from_secs(30)).await;
    }
    source_client.ack(message).await?;
    Ok(())
}

async fn run_mqtt5(config: Config) -> AppResult<()> {
    let qos = config.route.qos;
    let source = options_v5(&config.source, qos)?;
    let destination = options(&config.destination, false, false)?;
    let (source_client, mut source_loop) = v5::AsyncClient::new(source, 8);
    let (destination_client, destination_loop) = AsyncClient::new(destination, 8);
    let destination_connected = Arc::new(AtomicBool::new(false));
    let (ack_tx, mut ack_rx) = mpsc::channel(1);
    let mut destination_task: JoinHandle<AppResult<()>> = tokio::spawn(poll_destination(
        destination_loop,
        Arc::clone(&destination_connected),
        config.max_retries,
        qos,
        ack_tx,
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
                Ok(v5::Event::Incoming(v5::Incoming::ConnAck(connack))) => {
                    if connack.code != ConnectReturnCode::Success {
                        return Err(invalid("origem recusou CONNECT MQTT 5").into());
                    }
                    failures = 0;
                    tokio::select! {
                        result = &mut destination_task => return result?,
                        ready = timeout(Duration::from_secs(20), async {
                            while !destination_connected.load(Ordering::Acquire) {
                                sleep(Duration::from_millis(50)).await;
                            }
                        }) => ready.map_err(|_| invalid("destino indisponivel antes da assinatura MQTT 5"))?,
                    }
                    let subscription_qos = if qos == 1 { QoS5::AtLeastOnce } else { QoS5::AtMostOnce };
                    let mut subscription = Filter::new(&filter, subscription_qos);
                    subscription.preserve_retain = true;
                    source_client.subscribe_many([subscription]).await?;
                    eprintln!("Bridge: origem conectada");
                }
                Ok(v5::Event::Incoming(v5::Incoming::SubAck(suback))) => {
                    let expected = if qos == 1 { QoS5::AtLeastOnce } else { QoS5::AtMostOnce };
                    if !matches!(suback.return_codes.as_slice(),
                                 [SubscribeReasonCode::Success(actual)] if *actual == expected) {
                        return Err(invalid("origem recusou assinatura MQTT 5").into());
                    }
                    eprintln!("Bridge: origem assinada");
                }
                Ok(v5::Event::Incoming(v5::Incoming::Publish(message))) => {
                    if qos == 1 {
                        forward_mqtt5_qos1(&config, &message, &source_client,
                            &destination_client, &destination_connected, &mut ack_rx).await?;
                        forwarded += 1;
                        if forwarded.is_multiple_of(100) {
                            eprintln!("Bridge: {forwarded} mensagens encaminhadas");
                        }
                        continue;
                    }
                    let previous_dropped = dropped;
                    if !destination_connected.load(Ordering::Acquire)
                        || message.payload.len() > MAX_PAYLOAD_BYTES {
                        dropped += 1;
                    } else if let Ok(topic) = std::str::from_utf8(&message.topic)
                        && let Some(topic) = mapped_topic(&config.route, topic) {
                        destination_client.publish(topic, QoS::AtMostOnce,
                                                   message.retain, message.payload).await?;
                        forwarded += 1;
                        if forwarded.is_multiple_of(100) {
                            eprintln!("Bridge: {forwarded} mensagens encaminhadas");
                        }
                    } else {
                        dropped += 1;
                    }
                    if dropped != previous_dropped
                        && (dropped == 1 || dropped.is_multiple_of(100)) {
                        eprintln!("Bridge: {dropped} mensagens descartadas (destino indisponivel ou limite)");
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

async fn poll_destination(
    mut loopback: rumqttc::EventLoop,
    connected: Arc<AtomicBool>,
    retries: u8,
    qos: u8,
    ack_tx: mpsc::Sender<()>,
) -> AppResult<()> {
    let mut failures = 0_u8;
    loop {
        match loopback.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                connected.store(true, Ordering::Release);
                failures = 0;
                eprintln!("Bridge: destino conectado");
            }
            Ok(Event::Incoming(Incoming::PubAck(_))) if qos == 1 => {
                ack_tx.send(()).await?;
            }
            Ok(Event::Incoming(Incoming::PubComp(_))) if qos == 2 => {
                ack_tx.send(()).await?;
            }
            #[cfg(debug_assertions)]
            Ok(Event::Outgoing(Outgoing::Publish(_)))
                if qos == 2
                    && std::env::var_os("XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_PUBLISH").is_some() =>
            {
                eprintln!("Bridge: checkpoint apos PUBLISH do destino");
                sleep(Duration::from_secs(3)).await;
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

fn open_journal(config: &Config) -> AppResult<Option<Journal>> {
    if config.route.qos != 2 {
        return Ok(None);
    }
    let identity = serde_json::to_vec(&(
        &config.source.broker_id,
        &config.source.host,
        config.source.port,
        &config.source.client_id,
        &config.source.username,
        &config.destination.broker_id,
        &config.destination.host,
        config.destination.port,
        &config.destination.client_id,
        &config.destination.username,
        &config.route.source_prefix,
        &config.route.target_prefix,
        config.route.qos,
        &config.link_id,
    ))?;
    Ok(Some(Journal::open(
        config
            .journal_dir
            .as_deref()
            .ok_or_else(|| invalid("journal_dir ausente"))?,
        &identity,
    )?))
}

async fn forward_durable(
    config: &Config,
    message: &Publish,
    source_client: &AsyncClient,
    destination_client: &AsyncClient,
    destination_connected: &AtomicBool,
    ack_rx: &mut mpsc::Receiver<()>,
    journal: &mut Option<Journal>,
) -> AppResult<bool> {
    let expected_qos = if config.route.qos == 2 {
        QoS::ExactlyOnce
    } else {
        QoS::AtLeastOnce
    };
    if message.qos != expected_qos || message.payload.len() > MAX_PAYLOAD_BYTES {
        return Err(invalid("mensagem QoS invalida ou acima do limite").into());
    }
    let topic = mapped_topic(&config.route, &message.topic)
        .ok_or_else(|| invalid("topico de origem nao mapeavel"))?;
    if let Some(journal) = journal.as_ref()
        && journal.already_forwarded(
            message.pkid,
            &message.topic,
            &message.payload,
            message.retain,
        )?
    {
        source_client.ack(message).await?;
        return Ok(false);
    }
    timeout(Duration::from_secs(20), async {
        while !destination_connected.load(Ordering::Acquire) {
            sleep(Duration::from_millis(50)).await;
        }
        destination_client
            .publish(topic, expected_qos, message.retain, message.payload.clone())
            .await?;
        ack_rx
            .recv()
            .await
            .ok_or_else(|| invalid("destino sem ACK terminal"))?;
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    })
    .await
    .map_err(|_| invalid("destino sem ACK terminal no prazo"))??;
    if let Some(journal) = journal.as_mut() {
        journal.record(
            message.pkid,
            &message.topic,
            &message.payload,
            message.retain,
        )?;
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("XMQR_BRIDGE_TEST_PAUSE_AFTER_DEST_ACK").is_some()
        && message.topic.ends_with("/crash")
    {
        if config.route.qos == 2 {
            eprintln!("Bridge: checkpoint apos PUBCOMP do destino");
        } else {
            eprintln!("Bridge: checkpoint apos PUBACK do destino");
        }
        sleep(Duration::from_secs(30)).await;
    }
    source_client.ack(message).await?;
    Ok(true)
}

fn qos_level(qos: u8) -> QoS {
    match qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => unreachable!(),
    }
}

#[cfg(debug_assertions)]
async fn pause_after_source_pubrec(pkid: u16, journal: Option<&Journal>) {
    let test_suffix = match std::env::var("XMQR_BRIDGE_TEST_PAUSE_AFTER_SOURCE_PUBREC").as_deref() {
        Ok("1") => Some("/crash"),
        Ok("session-loss") => Some("/session-loss"),
        _ => None,
    };
    if test_suffix.is_some_and(|suffix| {
        journal
            .and_then(|journal| journal.pending_topic(pkid))
            .is_some_and(|topic| topic.ends_with(suffix))
    }) {
        eprintln!("Bridge: checkpoint apos PUBREC da origem");
        sleep(Duration::from_secs(30)).await;
    }
}

async fn run(config: Config) -> AppResult<()> {
    if config.source_protocol == SourceProtocol::Mqtt5 {
        return run_mqtt5(config).await;
    }
    let qos = config.route.qos;
    let durable_source = qos > 0;
    let mut journal = open_journal(&config)?;
    let source = options(&config.source, durable_source, durable_source)?;
    let destination = options(&config.destination, qos == 2, false)?;
    let (source_client, mut source_loop) = AsyncClient::new(source, 8);
    let (destination_client, destination_loop) = AsyncClient::new(destination, 8);
    let destination_connected = Arc::new(AtomicBool::new(false));
    let (ack_tx, mut ack_rx) = mpsc::channel(1);
    let mut destination_task: JoinHandle<AppResult<()>> = tokio::spawn(poll_destination(
        destination_loop,
        Arc::clone(&destination_connected),
        config.max_retries,
        qos,
        ack_tx,
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
                Ok(Event::Incoming(Incoming::ConnAck(connack))) => {
                    failures = 0;
                    if let Some(journal) = &journal {
                        if journal.has_pending() && !connack.session_present {
                            eprintln!("Bridge: sessao de origem ausente; journal QoS 2 preservado. Recupere a sessao original antes de reiniciar");
                            return Err(invalid("journal QoS 2 pendente sem sessao de origem").into());
                        }
                        for pkid in journal.pending_ids() {
                            source_loop.state.restore_incoming_qos2(pkid)?;
                        }
                    }
                    source_client.subscribe(&filter, qos_level(qos)).await?;
                    eprintln!("Bridge: origem conectada");
                }
                Ok(Event::Incoming(Incoming::SubAck(_))) => {
                    eprintln!("Bridge: origem assinada");
                }
                Ok(Event::Incoming(Incoming::Publish(message))) => {
                    if durable_source {
                        if forward_durable(&config, &message, &source_client,
                                           &destination_client, &destination_connected,
                                           &mut ack_rx, &mut journal).await? {
                            forwarded += 1;
                            if forwarded.is_multiple_of(100) {
                                eprintln!("Bridge: {forwarded} mensagens encaminhadas");
                            }
                        }
                        continue;
                    }
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
                Ok(Event::Incoming(Incoming::PubRel(pubrel))) if qos == 2 => {
                    journal.as_mut().ok_or_else(|| invalid("journal ausente"))?
                        .finish(pubrel.pkid)?;
                }
                #[cfg(debug_assertions)]
                Ok(Event::Outgoing(Outgoing::PubRec(pkid))) if qos == 2 => {
                    pause_after_source_pubrec(pkid, journal.as_ref()).await;
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
            "mqtt-bridge 0.3.0 (xmqr-client {})",
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "Uso: mqtt-bridge --config ARQUIVO_PRIVADO_TOML\nBridge experimental unidirecional TLS/mTLS QoS 0, 1 ou 2."
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
    use super::{Link, Route, SourceProtocol, has_cycle, mapped_topic, valid_topic};

    #[test]
    fn mqtt5_source_is_explicit_and_unknown_protocol_is_rejected() {
        assert_eq!(SourceProtocol::default(), SourceProtocol::Mqtt311);
        assert_eq!(
            serde_json::from_str::<SourceProtocol>("\"mqtt5\"").unwrap(),
            SourceProtocol::Mqtt5
        );
        assert!(serde_json::from_str::<SourceProtocol>("\"mqtt4\"").is_err());
    }

    #[test]
    fn prefix_mapping_stays_bounded_and_does_not_match_neighbors() {
        let route = Route {
            source_prefix: "sensor/".into(),
            target_prefix: "mirror/".into(),
            qos: 0,
        };
        assert_eq!(
            mapped_topic(&route, "sensor/room/temp").as_deref(),
            Some("mirror/room/temp")
        );
        assert_eq!(mapped_topic(&route, "sensors/room/temp"), None);
        assert_eq!(mapped_topic(&route, "sensor/room/#"), None);
        assert!(!valid_topic(&"x".repeat(1025)));
    }

    #[test]
    fn topology_rejects_direct_and_three_link_cycles_but_accepts_chain() {
        fn link(id: &str, from: &str, to: &str, source: &str, target: &str) -> Link {
            Link {
                id: id.into(),
                source_broker: from.into(),
                destination_broker: to.into(),
                source_prefix: source.into(),
                target_prefix: target.into(),
                qos: 1,
            }
        }
        let first = link("first", "a", "b", "sensors/", "mirror/");
        let second = link("second", "b", "c", "mirror/", "archive/");
        assert!(!has_cycle(&[first, second]));
        assert!(has_cycle(&[
            link("first", "a", "b", "sensors/", "mirror/"),
            link("reverse", "b", "a", "mirror/", "sensors/"),
        ]));
        assert!(has_cycle(&[
            link("first", "a", "b", "sensors/", "mirror/"),
            link("second", "b", "c", "mirror/", "archive/"),
            link("third", "c", "a", "archive/", "sensors/"),
        ]));
        assert!(has_cycle(&[
            link("first", "a", "b", "sensors/", "mirror/"),
            link("reverse", "b", "a", "mirror/room/", "sensors/"),
        ]));
    }
}
