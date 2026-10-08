use std::{error::Error, io, time::Duration};

use rumqttc::{
    AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions, Outgoing, QoS,
    Transport,
    mqttbytes::v4::{ConnectReturnCode, SubscribeReasonCode},
};
use tokio::time::{Instant, timeout, timeout_at};

use super::{
    cli::{Cli, Mode, TransportMode},
    tls,
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const INITIAL_TIMEOUT: Duration = Duration::from_secs(8);
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_PACKET_BYTES: usize = 64 * 1024;

pub async fn run(cli: Cli, password: Option<String>) -> AppResult<()> {
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
    options.set_keep_alive(Duration::from_secs(30));
    options.set_max_packet_size(MAX_PACKET_BYTES, MAX_PACKET_BYTES);
    if let (Some(username), Some(password)) = (cli.username.clone(), password) {
        // MQTT 3.1.1 §3.1.3.4-5: secure mode carries both credentials in CONNECT.
        options.set_credentials(username, password);
    }

    let (client, mut events) = AsyncClient::new(options, 8);
    let result = tokio::select! {
        result = run_connected(&cli, &client, &mut events) => return result,
        signal = tokio::signal::ctrl_c() => signal,
    };
    result?;
    cancel_connection(&client, &mut events).await;
    eprintln!("Cancelado por Ctrl+C");
    Ok(())
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

async fn run_connected(cli: &Cli, client: &AsyncClient, events: &mut EventLoop) -> AppResult<()> {
    let mut retry = Retry::new(cli.reconnect_attempts);
    await_connack(events, &mut retry).await?;

    let requested_qos = cli.qos;
    match &cli.mode {
        Mode::Pub { topic, message } => {
            client
                .publish(topic, requested_qos, cli.retain, message.as_bytes())
                .await?;
            await_publish_complete(events, requested_qos).await?;
            graceful_disconnect(client, events).await?;
            cli.output.complete(requested_qos);
        }
        Mode::Sub { filter, count } => {
            subscribe(
                client,
                events,
                &mut retry,
                filter,
                requested_qos,
                *count,
                cli.output,
            )
            .await?;
        }
    }
    Ok(())
}

async fn await_connack(events: &mut EventLoop, retry: &mut Retry) -> AppResult<()> {
    loop {
        match retry.next(events, INITIAL_TIMEOUT).await? {
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

async fn await_publish_complete(events: &mut EventLoop, qos: QoS) -> AppResult<()> {
    let deadline = Instant::now() + INITIAL_TIMEOUT;
    loop {
        match next_before(events, deadline, "envio de PUBLISH")
            .await
            .map_err(|_| {
                invalid_data("publicacao interrompida; resultado desconhecido, nao reenviada")
            })? {
            Event::Outgoing(Outgoing::Publish(0)) if qos == QoS::AtMostOnce => return Ok(()),
            Event::Incoming(Incoming::PubAck(_)) if qos == QoS::AtLeastOnce => return Ok(()),
            Event::Incoming(Incoming::PubComp(_)) if qos == QoS::ExactlyOnce => return Ok(()),
            Event::Outgoing(_) => {}
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
}

impl Retry {
    const fn new(attempts: u8) -> Self {
        Self {
            remaining: attempts,
            used: 0,
        }
    }

    fn take_delay(&mut self) -> Option<Duration> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let delay = Duration::from_millis((100_u64 << self.used).min(5000));
        self.used += 1;
        Some(delay)
    }

    async fn next(&mut self, events: &mut EventLoop, limit: Duration) -> AppResult<Event> {
        loop {
            let error = match timeout(limit, events.poll()).await {
                Ok(Ok(event)) => return Ok(event),
                Ok(Err(error)) => error,
                Err(_) => return Err(timed_out("resposta do broker").into()),
            };
            if !retryable(&error) {
                // Do not format NotConnAck(Packet): untrusted packets may carry secrets.
                return Err(invalid_data("falha MQTT/TLS nao recuperavel").into());
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
    filter: &str,
    qos: QoS,
    count: Option<u64>,
    output: crate::output::Output,
) -> AppResult<()> {
    client.subscribe(filter, qos).await?;
    let mut acknowledged = false;
    let mut waiting_ack = true;
    let mut deadline = Instant::now() + INITIAL_TIMEOUT;
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
            let duration = deadline.saturating_duration_since(Instant::now());
            retry.next(events, duration).await?
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
                    client.subscribe(filter, qos).await?;
                    waiting_ack = true;
                    acknowledged = false;
                } else {
                    waiting_ack = false;
                }
                deadline = Instant::now()
                    + if waiting_ack {
                        INITIAL_TIMEOUT
                    } else {
                        IDLE_TIMEOUT
                    };
            }
            Event::Incoming(Incoming::SubAck(ack)) if waiting_ack => {
                if !matches!(ack.return_codes.as_slice(),
                    [SubscribeReasonCode::Success(granted)] if *granted == qos)
                {
                    return Err(invalid_data("SUBACK recusou ou alterou QoS solicitado").into());
                }
                waiting_ack = false;
                acknowledged = true;
                deadline = Instant::now() + IDLE_TIMEOUT;
                output.status("Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.");
            }
            Event::Incoming(Incoming::Publish(publish)) => {
                if publish.qos == QoS::ExactlyOnce {
                    if incoming_qos2.is_empty() {
                        qos2_deadline = Some(Instant::now() + INITIAL_TIMEOUT);
                    }
                    incoming_qos2.insert(publish.pkid);
                }
                if count.is_none_or(|limit| received < limit) {
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
        if !waiting_ack {
            deadline = Instant::now() + IDLE_TIMEOUT;
        }
    }
    graceful_disconnect(client, events).await
}

async fn graceful_disconnect(client: &AsyncClient, events: &mut EventLoop) -> AppResult<()> {
    client.disconnect().await?;
    let deadline = Instant::now() + INITIAL_TIMEOUT;
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
    fn retry_budget_and_progressive_delays_are_finite() {
        let mut retry = Retry::new(10);
        let delays: Vec<_> = (0..10)
            .map(|_| retry.take_delay().unwrap().as_millis())
            .collect();
        assert_eq!(
            delays,
            [100, 200, 400, 800, 1600, 3200, 5000, 5000, 5000, 5000]
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
