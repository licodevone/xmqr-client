use std::{
    collections::hash_map::RandomState, error::Error, hash::BuildHasher, io, time::Duration,
};

use rumqttc::{
    AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions, Outgoing, QoS,
    Transport,
    mqttbytes::v4::{ConnectReturnCode, SubscribeFilter, SubscribeReasonCode},
};
use tokio::time::{Instant, timeout, timeout_at};

use super::{
    cli::{Cli, Mode, TransportMode},
    tls,
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_PACKET_BYTES: usize = 64 * 1024;
// rumqttc 0.25.1 checks incoming Remaining Length, outgoing full frame.
// Reserve the maximum five-byte header, including non-minimal length encodings.
// Canonical frames of exactly 64 KiB may be refused one byte early; documented in C38.
const MAX_INCOMING_REMAINING_BYTES: usize = MAX_PACKET_BYTES - 5;

pub async fn run(
    cli: Cli,
    password: Option<String>,
    publication: Option<crate::payload::Publication>,
) -> AppResult<()> {
    // One monotonic deadline; reconnects and ordinary traffic cannot renew it.
    // Initial payload/password reads occur before run() and are excluded.
    let total_deadline = cli
        .total_timeout_seconds
        .map(|seconds| Instant::now() + Duration::from_secs(u64::from(seconds)));
    let mut options = MqttOptions::new(&cli.client_id, &cli.host, cli.port);
    if cli.transport_mode == TransportMode::Secure {
        let tls_config = tls::build_client_config(&cli)?;
        options.set_transport(Transport::tls_with_config(tls_config.into()));
    } else {
        options.set_transport(Transport::Tcp);
    }
    if let Some(will) = &cli.will {
        options.set_last_will(rumqttc::LastWill::new(
            &will.topic,
            will.message.as_bytes(),
            will.qos,
            will.retain,
        ));
    }
    options.set_clean_session(cli.clean_session);
    options.set_keep_alive(Duration::from_secs(u64::from(cli.keepalive_seconds)));
    options.set_max_packet_size(MAX_INCOMING_REMAINING_BYTES, MAX_PACKET_BYTES);
    if let (Some(username), Some(password)) = (cli.username.clone(), password) {
        // MQTT 3.1.1 §3.1.3.4-5: secure mode carries both credentials in CONNECT.
        options.set_credentials(username, password);
    }

    let (client, mut events) = AsyncClient::new(options, 8);
    let result = tokio::select! {
        biased;
        signal = crate::signals::shutdown() => signal,
        () = total_expired(total_deadline) => {
            // Do not add a grace period or call poll() after total expiry.
            let message = if matches!(cli.mode, Mode::Pub { .. }) {
                "tempo total da operacao esgotado; publicacao pode ter resultado desconhecido, nao reenviada"
            } else {
                "tempo total da operacao esgotado"
            };
            return Err(io::Error::new(io::ErrorKind::TimedOut, message).into());
        },
        result = run_connected(&cli, &client, &mut events, publication) => return result,
    };
    result?;
    if let Some(deadline) = cancellation_deadline(total_deadline) {
        let _ = timeout_at(deadline, cancel_connection(&client, &mut events)).await;
    }
    eprintln!("Cancelado por sinal de encerramento");
    Ok(())
}

fn cancellation_deadline(total: Option<Instant>) -> Option<Instant> {
    let now = Instant::now();
    let normal = now + Duration::from_secs(1);
    let deadline = total.map_or(normal, |total| total.min(normal));
    (deadline > now).then_some(deadline)
}

async fn total_expired(deadline: Option<Instant>) {
    if let Some(deadline) = deadline {
        tokio::time::sleep_until(deadline).await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn cancel_connection(client: &AsyncClient, events: &mut EventLoop) {
    // Never reconnect while cancelling or drain pending application requests.
    // Dropping an unavailable connection is bounded; on a live link send MQTT DISCONNECT.
    if events.network.is_some() {
        let network = events.network.take();
        events.clean();
        events.pending.clear();
        events.network = network;
        let _ = timeout(Duration::from_secs(1), async {
            client.try_disconnect()?;
            loop {
                match events.poll().await {
                    Ok(Event::Outgoing(Outgoing::Disconnect)) | Err(_) => break,
                    Ok(_) => {}
                }
            }
            Ok::<(), rumqttc::ClientError>(())
        })
        .await;
    }
}

fn protocol_timeout(cli: &Cli) -> Duration {
    Duration::from_secs(u64::from(cli.timeout_seconds))
}

async fn run_connected(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut EventLoop,
    publication: Option<crate::payload::Publication>,
) -> AppResult<()> {
    let mut retry = Retry::new(cli.reconnect_attempts);
    await_connack(events, &mut retry, protocol_timeout(cli)).await?;

    let requested_qos = cli.qos;
    match &cli.mode {
        Mode::Pub { topic, .. } => {
            let publication =
                publication.ok_or_else(|| invalid_data("entrada de publicacao ausente"))?;
            publish_input(cli, client, events, topic, publication).await?;
        }
        Mode::Sub { filters, count } => {
            subscribe(
                client,
                events,
                &mut retry,
                filters,
                requested_qos,
                *count,
                cli.output,
                cli.retained_filter,
                protocol_timeout(cli),
            )
            .await?;
        }
    }
    Ok(())
}

async fn publish_input(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut EventLoop,
    topic: &str,
    publication: crate::payload::Publication,
) -> AppResult<()> {
    let crate::payload::Publication { mut input, first } = publication;
    let mut payload = Some(first);
    while let Some(bytes) = payload {
        client.publish(topic, cli.qos, cli.retain, bytes).await?;
        await_publish_complete(events, cli.qos, protocol_timeout(cli)).await?;
        if !cli.line_mode {
            break;
        }
        cli.output.complete(cli.qos);
        // Never retry a connection or replay a batch after the first PUBLISH.
        // Poll while waiting for input so keepalive remains active.
        let deadline = Instant::now() + crate::payload::INPUT_TIMEOUT;
        payload = loop {
            tokio::select! {
                result = input.next_before(deadline) => break result?,
                event = events.poll() => match event {
                    Ok(Event::Outgoing(_) | Event::Incoming(Incoming::PingResp)) => {},
                    Ok(_) => return Err(invalid_data("resposta MQTT inesperada entre mensagens").into()),
                    Err(_) => return Err(io::Error::new(io::ErrorKind::ConnectionAborted,
                        "conexao interrompida entre mensagens; lote parcial, nao reenviado").into()),
                }
            }
        };
    }
    graceful_disconnect(client, events, protocol_timeout(cli)).await?;
    if !cli.line_mode {
        cli.output.complete(cli.qos);
    }
    Ok(())
}

async fn await_connack(
    events: &mut EventLoop,
    retry: &mut Retry,
    limit: Duration,
) -> AppResult<()> {
    loop {
        match retry.next(events, limit, limit).await? {
            Event::Incoming(Incoming::ConnAck(ack)) if ack.code == ConnectReturnCode::Success => {
                return Ok(());
            }
            Event::Incoming(Incoming::ConnAck(_)) => {
                return Err(invalid_data("broker recusou CONNECT").into());
            }
            Event::Outgoing(_) => {}
            Event::Incoming(_) => {
                return Err(invalid_data("pacote recebido antes de CONNACK").into());
            }
        }
    }
}

async fn await_publish_complete(
    events: &mut EventLoop,
    qos: QoS,
    limit: Duration,
) -> AppResult<()> {
    let deadline = Instant::now() + limit;
    loop {
        match next_before(events, deadline, "envio de PUBLISH")
            .await
            .map_err(|_| {
                invalid_data("publicacao interrompida; resultado desconhecido, nao reenviada")
            })? {
            Event::Outgoing(Outgoing::Publish(0)) if qos == QoS::AtMostOnce => return Ok(()),
            Event::Incoming(Incoming::PubAck(_)) if qos == QoS::AtLeastOnce => return Ok(()),
            Event::Incoming(Incoming::PubComp(_)) if qos == QoS::ExactlyOnce => return Ok(()),
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp) => {}
            Event::Incoming(Incoming::PubRec(_)) if qos == QoS::ExactlyOnce => {}
            Event::Incoming(_) => return Err(invalid_data("resposta MQTT inesperada").into()),
        }
    }
}

// A single process-wide budget bounds reconnects even when a peer repeatedly
// accepts CONNECT and then closes. Reuse rumqttc's event loop, never rebuild it.
struct Retry {
    remaining: u8,
    used: u8,
    jitter_state: u64,
}

impl Retry {
    fn new(attempts: u8) -> Self {
        // RandomState provides a per-process seed; jitter is not a security primitive.
        Self::with_seed(attempts, RandomState::new().hash_one(std::process::id()))
    }

    const fn with_seed(attempts: u8, seed: u64) -> Self {
        Self {
            remaining: attempts,
            used: 0,
            jitter_state: if seed == 0 { 1 } else { seed },
        }
    }

    fn take_delay(&mut self) -> Option<Duration> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let base_ms = (100_u64 << self.used).min(5000);
        self.jitter_state ^= self.jitter_state << 13;
        self.jitter_state ^= self.jitter_state >> 7;
        self.jitter_state ^= self.jitter_state << 17;
        let percent = 80 + self.jitter_state % 21;
        let delay = Duration::from_millis(base_ms * percent / 100);
        self.used += 1;
        Some(delay)
    }

    async fn next(
        &mut self,
        events: &mut EventLoop,
        limit: Duration,
        connect_limit: Duration,
    ) -> AppResult<Event> {
        loop {
            let poll_limit = if events.network.is_none() {
                limit.min(connect_limit)
            } else {
                limit
            };
            let error = match timeout(poll_limit, events.poll()).await {
                Ok(Ok(event)) => return Ok(event),
                Ok(Err(error)) => error,
                Err(_) => return Err(timed_out("resposta do broker").into()),
            };
            if !retryable(&error) {
                // Do not format NotConnAck(Packet): untrusted packets may carry secrets.
                return Err(invalid_data(connection_diagnostic(&error)).into());
            }
            self.wait().await?;
        }
    }

    async fn wait(&mut self) -> AppResult<()> {
        let Some(delay) = self.take_delay() else {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "conexao interrompida; limite de reconexao esgotado",
            )
            .into());
        };
        eprintln!(
            "Reconexao {}/{} em {}ms",
            self.used,
            self.used + self.remaining,
            delay.as_millis()
        );
        tokio::time::sleep(delay).await;
        Ok(())
    }

    async fn next_before(
        &mut self,
        events: &mut EventLoop,
        deadline: Instant,
        connect_limit: Duration,
        waiting_ack: bool,
    ) -> AppResult<Event> {
        let duration = deadline.saturating_duration_since(Instant::now());
        // Absolute protocol/idle deadline includes failed polls and backoff.
        timeout_at(deadline, self.next(events, duration, connect_limit))
            .await
            .map_err(|_| {
                timed_out(if waiting_ack {
                    "SUBACK"
                } else {
                    "mensagens de aplicacao"
                })
            })?
    }
}

fn connection_diagnostic(error: &ConnectionError) -> &'static str {
    match error {
        ConnectionError::ConnectionRefused(ConnectReturnCode::BadUserNamePassword) => {
            "autenticacao MQTT recusada; confira usuario e senha"
        }
        ConnectionError::ConnectionRefused(ConnectReturnCode::NotAuthorized) => {
            "acesso MQTT recusado; confira identidade e politica do broker"
        }
        ConnectionError::ConnectionRefused(ConnectReturnCode::RefusedProtocolVersion) => {
            "versao MQTT recusada; este client usa MQTT 3.1.1"
        }
        ConnectionError::ConnectionRefused(ConnectReturnCode::BadClientId) => {
            "ClientID recusado pelo broker"
        }
        ConnectionError::ConnectionRefused(ConnectReturnCode::ServiceUnavailable) => {
            "broker MQTT indisponivel"
        }
        ConnectionError::MqttState(rumqttc::StateError::Deserialization(
            rumqttc::mqttbytes::Error::PayloadSizeLimitExceeded(_),
        )) => {
            "pacote MQTT recebido excede limite local de recepcao: corpo ate 65531 bytes, pacote ate 64 KiB"
        }
        ConnectionError::Tls(_) => {
            "falha TLS; confira CA, SAN, validade e certificado/chave do client"
        }
        ConnectionError::Io(_) => "falha de transporte nao recuperavel; confira rede e TLS",
        _ => "falha MQTT/TLS nao recuperavel",
    }
}

fn retryable(error: &ConnectionError) -> bool {
    match error {
        ConnectionError::NetworkTimeout
        | ConnectionError::FlushTimeout
        | ConnectionError::MqttState(
            rumqttc::StateError::ConnectionAborted | rumqttc::StateError::AwaitPingResp,
        ) => true,
        ConnectionError::Io(error)
        | ConnectionError::MqttState(
            rumqttc::StateError::Io(error)
            | rumqttc::StateError::Deserialization(rumqttc::mqttbytes::Error::Io(error)),
        ) => matches!(
            error.kind(),
            io::ErrorKind::ConnectionRefused
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::UnexpectedEof
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::TimedOut
                | io::ErrorKind::NotConnected
                | io::ErrorKind::WouldBlock
        ),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
async fn subscribe(
    client: &AsyncClient,
    events: &mut EventLoop,
    retry: &mut Retry,
    filters: &[String],
    qos: QoS,
    count: Option<u64>,
    output: crate::output::Output,
    retained_filter: crate::cli::RetainedFilter,
    limit: Duration,
) -> AppResult<()> {
    client
        .subscribe_many(
            filters
                .iter()
                .cloned()
                .map(|filter| SubscribeFilter::new(filter, qos)),
        )
        .await?;
    let mut acknowledged = false;
    let mut waiting_ack = true;
    let mut deadline = Instant::now() + limit;
    let mut received = 0;
    let mut incoming_qos2 = std::collections::HashSet::new();
    let mut qos2_deadline = None;
    loop {
        // Count may be satisfied before SUBACK; still validate the subscription.
        if !waiting_ack && incoming_qos2.is_empty() && count.is_some_and(|limit| received >= limit)
        {
            break;
        }
        let event = if incoming_qos2.is_empty() {
            retry
                .next_before(events, deadline, limit, waiting_ack)
                .await?
        } else {
            // rumqttc clean() drops incoming QoS2 state. Resuming a partially
            // received handshake could reject PUBREL or duplicate application data.
            next_before(events, qos2_deadline.unwrap(), "conclusao QoS2 recebido")
                .await
                .map_err(|_| {
                    invalid_data("recepcao QoS2 interrompida; retomada do handshake nao suportada")
                })?
        };
        match event {
            Event::Incoming(Incoming::ConnAck(ack)) => {
                incoming_qos2.clear();
                if ack.code != ConnectReturnCode::Success {
                    return Err(invalid_data("broker recusou CONNECT").into());
                }
                if !ack.session_present || !acknowledged {
                    // clean() can preserve an unsent SUBSCRIBE. Remove only that
                    // request before replacing it; keep protocol QoS ACK state.
                    events
                        .pending
                        .retain(|request| !matches!(request, rumqttc::Request::Subscribe(_)));
                    client
                        .subscribe_many(
                            filters
                                .iter()
                                .cloned()
                                .map(|filter| SubscribeFilter::new(filter, qos)),
                        )
                        .await?;
                    waiting_ack = true;
                    acknowledged = false;
                } else {
                    waiting_ack = false;
                }
                deadline = Instant::now() + if waiting_ack { limit } else { IDLE_TIMEOUT };
            }
            Event::Incoming(Incoming::SubAck(ack)) if waiting_ack => {
                if ack.return_codes.len() != filters.len()
                    || ack.return_codes.iter().any(|code| {
                        !matches!(code, SubscribeReasonCode::Success(granted) if *granted == qos)
                    })
                {
                    return Err(invalid_data("SUBACK recusou ou alterou QoS solicitado; confira ACL e limites do broker; MQTT 3.1.1 nao informa causa especifica").into());
                }
                waiting_ack = false;
                acknowledged = true;
                deadline = Instant::now() + IDLE_TIMEOUT;
                output.status("Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.");
            }
            Event::Incoming(Incoming::Publish(publish)) => {
                if publish.qos == QoS::ExactlyOnce {
                    if incoming_qos2.is_empty() {
                        qos2_deadline = Some(Instant::now() + limit);
                    }
                    incoming_qos2.insert(publish.pkid);
                }
                if retained_filter.accepts(publish.retain)
                    && count.is_none_or(|limit| received < limit)
                {
                    output.publish(&publish);
                    received += 1;
                }
                if !waiting_ack {
                    deadline = Instant::now() + IDLE_TIMEOUT;
                }
            }
            Event::Incoming(Incoming::PubRel(release)) => {
                incoming_qos2.remove(&release.pkid);
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp) => {}
            Event::Incoming(_) => return Err(invalid_data("resposta MQTT inesperada").into()),
        }
        // PINGRESP and outbound protocol ACKs do not renew application idle.
    }
    graceful_disconnect(client, events, limit).await
}

async fn graceful_disconnect(
    client: &AsyncClient,
    events: &mut EventLoop,
    limit: Duration,
) -> AppResult<()> {
    client.disconnect().await?;
    let deadline = Instant::now() + limit;
    loop {
        match next_before(events, deadline, "DISCONNECT").await? {
            Event::Outgoing(Outgoing::Disconnect) => return Ok(()),
            Event::Incoming(_) | Event::Outgoing(_) => {}
        }
    }
}

async fn next_before(
    events: &mut EventLoop,
    deadline: Instant,
    stage: &'static str,
) -> AppResult<Event> {
    timeout_at(deadline, events.poll())
        .await
        .map_err(|_| timed_out(stage))?
        .map_err(Into::into)
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn timed_out(stage: &'static str) -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!("tempo esgotado aguardando {stage}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_budget_never_extends_total_deadline() {
        let now = Instant::now();
        assert!(cancellation_deadline(None).unwrap() <= Instant::now() + Duration::from_secs(1));
        let short = now + Duration::from_millis(25);
        assert_eq!(cancellation_deadline(Some(short)), Some(short));
        assert_eq!(cancellation_deadline(Some(now)), None);
        assert!(
            cancellation_deadline(Some(now + Duration::from_secs(10))).unwrap()
                < now + Duration::from_secs(2)
        );
    }

    #[test]
    fn diagnostics_distinguish_authentication_and_protocol_without_error_contents() {
        assert!(
            connection_diagnostic(&ConnectionError::ConnectionRefused(
                ConnectReturnCode::BadUserNamePassword
            ))
            .contains("autenticacao")
        );
        assert!(
            connection_diagnostic(&ConnectionError::ConnectionRefused(
                ConnectReturnCode::NotAuthorized
            ))
            .contains("acesso")
        );
        assert!(
            connection_diagnostic(&ConnectionError::ConnectionRefused(
                ConnectReturnCode::RefusedProtocolVersion
            ))
            .contains("3.1.1")
        );
        let secret_error = ConnectionError::Io(io::Error::other("secret-fixture-content"));
        assert!(!connection_diagnostic(&secret_error).contains("secret-fixture-content"));
    }

    #[test]
    fn retry_budget_and_progressive_delays_are_finite() {
        let mut retry = Retry::with_seed(10, 123);
        let delays: Vec<_> = (0..10)
            .map(|_| retry.take_delay().unwrap().as_millis())
            .collect();
        let bases = [100, 200, 400, 800, 1600, 3200, 5000, 5000, 5000, 5000];
        for (delay, base) in delays.iter().zip(bases) {
            assert!((base * 80 / 100..=base).contains(delay));
        }
        let mut same_seed = Retry::with_seed(10, 123);
        assert_eq!(
            delays,
            (0..10)
                .map(|_| same_seed.take_delay().unwrap().as_millis())
                .collect::<Vec<_>>()
        );
        assert_eq!(retry.take_delay(), None);
        assert_eq!(Retry::new(0).take_delay(), None);
    }

    #[test]
    fn only_transport_errors_retry() {
        assert!(retryable(&ConnectionError::MqttState(
            rumqttc::StateError::ConnectionAborted
        )));
        assert!(retryable(&ConnectionError::Io(
            io::ErrorKind::ConnectionRefused.into()
        )));
        assert!(!retryable(&ConnectionError::ConnectionRefused(
            ConnectReturnCode::NotAuthorized
        )));
        assert!(!retryable(&ConnectionError::MqttState(
            rumqttc::StateError::Unsolicited(7)
        )));
    }

    #[tokio::test]
    async fn cancellation_sends_disconnect_on_live_link() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut header = [0; 2];
            stream.read_exact(&mut header).await.unwrap();
            assert_eq!(header[0], 0x10);
            let mut body = vec![0; usize::from(header[1])];
            stream.read_exact(&mut body).await.unwrap();
            stream.write_all(&[0x20, 2, 0, 0]).await.unwrap();
            stream.read_exact(&mut header).await.unwrap();
            assert_eq!(header, [0xe0, 0]);
        });
        let (client, mut events) =
            AsyncClient::new(MqttOptions::new("cancel", "127.0.0.1", port), 8);
        assert!(matches!(
            events.poll().await.unwrap(),
            Event::Incoming(Incoming::ConnAck(_))
        ));
        client
            .try_publish("discard", QoS::AtMostOnce, false, "queued")
            .unwrap();
        timeout(
            Duration::from_secs(2),
            cancel_connection(&client, &mut events),
        )
        .await
        .unwrap();
        timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn cancellation_drops_retry_backoff_without_new_connection() {
        let (client, mut events) =
            AsyncClient::new(MqttOptions::new("cancel-retry", "127.0.0.1", 1883), 8);
        // Test the actual backoff future directly: no dependence on how quickly
        // Windows reports a refused TCP connection and no socket can be opened.
        let mut retry = Retry {
            remaining: 1,
            used: 5,
            jitter_state: 123,
        };
        assert!(
            timeout(Duration::from_millis(100), retry.wait())
                .await
                .is_err()
        );
        assert_eq!(retry.used, 6);
        assert_eq!(retry.remaining, 0);
        assert!(events.network.is_none());
        timeout(
            Duration::from_millis(100),
            cancel_connection(&client, &mut events),
        )
        .await
        .unwrap();
        assert!(events.network.is_none());
    }
}
