// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
// New C27 contract reconstruction, not copied production implementation.
use crate::{
    args::{Link, Settings},
    format, security,
};
use rumqttc::{
    AsyncClient, ConnectionError, Event, EventLoop, Incoming, MqttOptions, Outgoing, QoS, Request,
    StateError, Transport,
    mqttbytes::{
        self,
        v4::{ConnectReturnCode, SubscribeReasonCode},
    },
};
use std::{collections::HashSet, io, time::Duration};
use tokio::time::{Instant, timeout, timeout_at};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const ACK_WAIT: Duration = Duration::from_secs(8);
const IDLE_WAIT: Duration = Duration::from_secs(90);

fn problem(message: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(io::Error::other(message.to_owned()))
}

struct Budget {
    maximum: u8,
    spent: u8,
}
impl Budget {
    fn reserve(&mut self) -> Option<Duration> {
        if self.spent >= self.maximum {
            return None;
        }
        let milliseconds = (100_u64 << self.spent).min(5000);
        self.spent += 1;
        Some(Duration::from_millis(milliseconds))
    }
    async fn pause(&mut self) -> Result<()> {
        let wait = self
            .reserve()
            .ok_or_else(|| problem("conexao interrompida; limite de reconexao esgotado"))?;
        eprintln!(
            "Reconexao {}/{} em {}ms",
            self.spent,
            self.maximum,
            wait.as_millis()
        );
        tokio::time::sleep(wait).await;
        Ok(())
    }
}

fn transient_io(kind: io::ErrorKind) -> bool {
    matches!(
        kind,
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::UnexpectedEof
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::TimedOut
            | io::ErrorKind::NotConnected
            | io::ErrorKind::WouldBlock
    )
}
fn transient(error: &ConnectionError) -> bool {
    match error {
        ConnectionError::Io(e)
        | ConnectionError::MqttState(
            StateError::Io(e) | StateError::Deserialization(mqttbytes::Error::Io(e)),
        ) => transient_io(e.kind()),
        ConnectionError::NetworkTimeout
        | ConnectionError::FlushTimeout
        | ConnectionError::MqttState(StateError::ConnectionAborted | StateError::AwaitPingResp) => {
            true
        }
        _ => false,
    }
}

struct Driver<'a> {
    settings: &'a Settings,
    client: AsyncClient,
    events: EventLoop,
    budget: Budget,
}

pub async fn start(settings: &Settings, password: Option<String>) -> Result<()> {
    let mut options = MqttOptions::new(&settings.id, &settings.host, settings.port);
    if settings.link == Link::Secure {
        options.set_transport(Transport::tls_with_config(security::tls(settings)?.into()));
    } else {
        options.set_transport(Transport::Tcp);
    }
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_session(settings.clean);
    options.set_max_packet_size(64 * 1024, 64 * 1024);
    if let Some(will) = &settings.testament {
        options.set_last_will(rumqttc::LastWill::new(
            &will.topic,
            will.text.as_bytes(),
            will.qos,
            will.retained,
        ));
    }
    if let (Some(user), Some(password)) = (&settings.user, password) {
        options.set_credentials(user, password);
    }
    let (client, events) = AsyncClient::new(options, 8);
    let mut driver = Driver {
        settings,
        client,
        events,
        budget: Budget {
            maximum: settings.retries,
            spent: 0,
        },
    };
    let signal = tokio::select! {
        result=driver.execute()=>return result,
        signal=tokio::signal::ctrl_c()=>signal,
    };
    signal?;
    driver.cancel().await;
    eprintln!("Cancelado por Ctrl+C");
    Ok(())
}

impl Driver<'_> {
    async fn recover(&mut self, limit: Duration) -> Result<Event> {
        loop {
            let polled = timeout(limit, self.events.poll())
                .await
                .map_err(|_| problem("tempo esgotado aguardando resposta do broker"))?;
            match polled {
                Ok(event) => return Ok(event),
                Err(error) if transient(&error) => self.budget.pause().await?,
                Err(_) => return Err(problem("falha MQTT/TLS nao recuperavel")),
            }
        }
    }
    async fn once(&mut self, deadline: Instant) -> Result<Event> {
        timeout_at(deadline, self.events.poll())
            .await
            .map_err(|_| problem("tempo esgotado aguardando pacote MQTT"))?
            .map_err(|_| problem("conexao/protocolo MQTT interrompido"))
    }
    async fn execute(&mut self) -> Result<()> {
        loop {
            match self.recover(ACK_WAIT).await? {
                Event::Incoming(Incoming::ConnAck(ack))
                    if ack.code == ConnectReturnCode::Success =>
                {
                    break;
                }
                Event::Outgoing(_) => {}
                Event::Incoming(_) => return Err(problem("CONNACK invalido")),
            }
        }
        if self.settings.publisher {
            self.publish().await?;
        } else {
            self.subscribe().await?;
        }
        self.disconnect().await?;
        if self.settings.publisher {
            println!(
                "{}",
                format::completed(self.settings.json(), self.settings.qos)
            );
        }
        Ok(())
    }
    async fn publish(&mut self) -> Result<()> {
        let c = self.settings;
        self.client
            .publish(
                &c.topic,
                c.qos,
                c.retained,
                c.text
                    .as_ref()
                    .ok_or_else(|| problem("payload ausente"))?
                    .as_bytes(),
            )
            .await?;
        let end = Instant::now() + ACK_WAIT;
        loop {
            let event = self.once(end).await.map_err(|_| {
                problem("publicacao interrompida; resultado desconhecido, nao reenviada")
            })?;
            match event {
                Event::Outgoing(Outgoing::Publish(0)) if c.qos == QoS::AtMostOnce => return Ok(()),
                Event::Incoming(Incoming::PubAck(_)) if c.qos == QoS::AtLeastOnce => return Ok(()),
                Event::Incoming(Incoming::PubComp(_)) if c.qos == QoS::ExactlyOnce => return Ok(()),
                Event::Incoming(Incoming::PubRec(_)) if c.qos == QoS::ExactlyOnce => {}
                Event::Outgoing(_) => {}
                Event::Incoming(_) => return Err(problem("resposta MQTT inesperada")),
            }
        }
    }
    #[allow(clippy::too_many_lines)] // Single subscription state machine includes QoS2 completion.
    async fn subscribe(&mut self) -> Result<()> {
        self.client
            .subscribe(&self.settings.topic, self.settings.qos)
            .await?;
        let mut waiting = true;
        let mut confirmed = false;
        let mut delivered = 0_u64;
        let mut qos2_ids = HashSet::new();
        let mut end = Instant::now() + ACK_WAIT;
        let mut qos2_end = Instant::now();
        loop {
            if !waiting
                && qos2_ids.is_empty()
                && self.settings.count.is_some_and(|n| delivered >= n)
            {
                return Ok(());
            }
            let event = if qos2_ids.is_empty() {
                self.recover(end.saturating_duration_since(Instant::now()))
                    .await?
            } else {
                self.once(qos2_end).await.map_err(|_| {
                    problem("recepcao QoS2 interrompida; retomada do handshake nao suportada")
                })?
            };
            match event {
                Event::Incoming(Incoming::ConnAck(ack)) => {
                    if ack.code != ConnectReturnCode::Success {
                        return Err(problem("CONNECT recusado"));
                    }
                    if !ack.session_present || !confirmed {
                        self.events
                            .pending
                            .retain(|r| !matches!(r, Request::Subscribe(_)));
                        self.client
                            .subscribe(&self.settings.topic, self.settings.qos)
                            .await?;
                        waiting = true;
                        confirmed = false;
                    } else {
                        waiting = false;
                    }
                    end = Instant::now() + if waiting { ACK_WAIT } else { IDLE_WAIT };
                }
                Event::Incoming(Incoming::SubAck(ack)) if waiting => {
                    if !matches!(ack.return_codes.as_slice(),[SubscribeReasonCode::Success(q)] if *q==self.settings.qos)
                    {
                        return Err(problem("SUBACK recusou ou alterou QoS solicitado"));
                    }
                    waiting = false;
                    confirmed = true;
                    format::active(self.settings.json());
                }
                Event::Incoming(Incoming::Publish(p)) => {
                    if p.qos == QoS::ExactlyOnce {
                        if qos2_ids.is_empty() {
                            qos2_end = Instant::now() + ACK_WAIT;
                        }
                        qos2_ids.insert(p.pkid);
                    }
                    if self.settings.count.is_none_or(|n| delivered < n) {
                        println!("{}", format::received(self.settings.json(), &p));
                        delivered += 1;
                    }
                }
                Event::Incoming(Incoming::PubRel(p)) => {
                    qos2_ids.remove(&p.pkid);
                }
                Event::Incoming(Incoming::PingResp) | Event::Outgoing(_) => {}
                Event::Incoming(_) => return Err(problem("resposta MQTT inesperada")),
            }
            if !waiting {
                end = Instant::now() + IDLE_WAIT;
            }
        }
    }
    async fn disconnect(&mut self) -> Result<()> {
        self.client.disconnect().await?;
        let deadline = Instant::now() + ACK_WAIT;
        loop {
            if matches!(
                self.once(deadline).await?,
                Event::Outgoing(Outgoing::Disconnect)
            ) {
                return Ok(());
            }
        }
    }
    async fn cancel(&mut self) {
        if self.events.network.is_none() {
            return;
        }
        let network = self.events.network.take();
        self.events.clean();
        self.events.pending.clear();
        self.events.network = network;
        let _ = timeout(Duration::from_secs(1), async {
            if self.client.try_disconnect().is_err() {
                return;
            }
            loop {
                match self.events.poll().await {
                    Ok(Event::Outgoing(Outgoing::Disconnect)) | Err(_) => return,
                    Ok(_) => {}
                }
            }
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_policy_and_budget_are_bounded() {
        let mut b = Budget {
            maximum: 10,
            spent: 0,
        };
        assert_eq!(
            (0..10)
                .map(|_| b.reserve().unwrap().as_millis())
                .collect::<Vec<_>>(),
            [100, 200, 400, 800, 1600, 3200, 5000, 5000, 5000, 5000]
        );
        assert!(b.reserve().is_none());
        assert!(!transient(&ConnectionError::ConnectionRefused(
            ConnectReturnCode::NotAuthorized
        )));
        assert!(!transient(&ConnectionError::MqttState(
            StateError::Unsolicited(1)
        )));
        assert!(transient(&ConnectionError::MqttState(
            StateError::ConnectionAborted
        )));
    }
    #[tokio::test]
    async fn dropping_backoff_is_cancellable_without_network() {
        let mut budget = Budget {
            maximum: 6,
            spent: 5,
        };
        assert!(
            timeout(Duration::from_millis(30), budget.pause())
                .await
                .is_err()
        );
        assert_eq!(budget.spent, 6);
    }

    #[tokio::test]
    async fn cancel_live_link_sends_disconnect_and_discards_queued_payload() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut header = [0; 2];
            socket.read_exact(&mut header).await.unwrap();
            assert_eq!(header[0], 0x10);
            let mut body = vec![0; usize::from(header[1])];
            socket.read_exact(&mut body).await.unwrap();
            socket.write_all(&[0x20, 2, 0, 0]).await.unwrap();
            socket.read_exact(&mut header).await.unwrap();
            assert_eq!(header, [0xe0, 0]);
        });
        let settings =
            crate::args::parse(["sub", "--open-lab", "--topic", "t"].map(str::to_owned)).unwrap();
        let (client, events) =
            AsyncClient::new(MqttOptions::new("cancel-check", "127.0.0.1", port), 8);
        let mut d = Driver {
            settings: &settings,
            client,
            events,
            budget: Budget {
                maximum: 0,
                spent: 0,
            },
        };
        timeout(Duration::from_secs(2), async {
            assert!(matches!(
                d.events.poll().await.unwrap(),
                Event::Incoming(Incoming::ConnAck(_))
            ));
            d.client
                .try_publish("discard", QoS::AtMostOnce, false, "never-send")
                .unwrap();
            d.cancel().await;
            server.await.unwrap();
        })
        .await
        .unwrap();
    }
}
