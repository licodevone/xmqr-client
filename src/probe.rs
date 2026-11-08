use std::{collections::hash_map::RandomState, hash::BuildHasher, io, sync::Arc, time::Duration};

use rumqttc::{Outgoing as ClientPacket, QoS, mqttbytes::v4, v5};
use serde_json::json;
use tokio::time::{Instant, timeout_at};

use crate::cli::{self, Cli, ProtocolVersion, TransportMode};

pub const USAGE: &str =
    "Diagnóstico online: mqtt-client probe [--topic TOPICO [--round-trip]] [opções de conexão]\n";

pub(crate) struct Probe {
    pub(crate) cli: Arc<Cli>,
    pub(crate) topic: Option<String>,
    pub(crate) marker: Option<Vec<u8>>,
}

pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Result<Probe, ()> {
    let id = format!("probe{:016x}", RandomState::new().hash_one(0_u8));
    let mut shared = vec![
        "sub".into(),
        "--client-id".into(),
        id,
        "--qos".into(),
        "0".into(),
        "--output".into(),
        "jsonl".into(),
    ];
    let mut topic = None;
    let mut round_trip = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--open-lab" | "--tls-password" | "--allow-remote-wss" => shared.push(flag),
            "--round-trip" if !round_trip => round_trip = true,
            "--topic" if topic.is_none() => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or(())?;
                if value.contains(['+', '#']) || value.starts_with("$share/") {
                    return Err(());
                }
                topic = Some(value);
            }
            "--profile" | "--host" | "--port" | "--ws-url" | "--ca" | "--cert" | "--key"
            | "--username" | "--password-file" | "--protocol-version" | "--timeout-seconds" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or(())?;
                shared.extend([flag, value]);
            }
            _ => return Err(()),
        }
    }
    if round_trip && topic.is_none() {
        return Err(());
    }
    shared.extend([
        "--topic".into(),
        topic.clone().unwrap_or_else(|| "xmqr/probe".into()),
    ]);
    let mut cli = cli::parse(shared).map_err(|_| ())?;
    cli.clean_session = true;
    let marker = round_trip
        .then(|| format!("xmqr-probe:{:016x}", RandomState::new().hash_one(0_u8)).into_bytes());
    Ok(Probe {
        cli: Arc::new(cli),
        topic,
        marker,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Failure {
    pub(crate) phase: &'static str,
    pub(crate) reason: &'static str,
}

type Diagnosis<T> = Result<T, Failure>;

const CONNECT: &str = "connect";
const SUBSCRIBE: &str = "subscribe";
const ROUND_TRIP: &str = "round_trip";

pub async fn run(mut args: impl Iterator<Item = String>) -> i32 {
    let first = args.next();
    if matches!(first.as_deref(), Some("--help" | "-h")) && args.next().is_none() {
        print!("{USAGE}");
        return 0;
    }
    let Ok(probe) = parse(first.into_iter().chain(args)) else {
        report(None, "configuration", "invalid_options", None);
        return 2;
    };
    let protocol = match probe.cli.protocol_version {
        ProtocolVersion::Mqtt311 => "3.1.1",
        ProtocolVersion::Mqtt5 => "5",
    };
    let password = if probe.cli.transport_mode == TransportMode::OpenLab {
        None
    } else {
        let Ok(password) = crate::read_mqtt_password(&probe.cli) else {
            report(Some(protocol), "credentials", "unavailable", None);
            return 2;
        };
        if cli::validate_password(&password).is_err() {
            report(Some(protocol), "credentials", "invalid", None);
            return 2;
        }
        Some(password)
    };
    let Ok(mut connection) = Connection::new(&probe.cli, password) else {
        report(Some(protocol), "tls", "invalid_materials", None);
        return 2;
    };
    let start = Instant::now();
    let seconds = u32::from(probe.cli.timeout_seconds);
    let seconds = seconds.min(probe.cli.total_timeout_seconds.unwrap_or(seconds));
    let deadline = start + Duration::from_secs(u64::from(seconds));
    let result = tokio::select! {
        biased;
        _ = crate::signals::shutdown() => Err(Failure { phase: "probe", reason: "cancelled" }),
        result = connection.diagnose(&probe, deadline) => result,
    };
    let elapsed = Some(start.elapsed().as_millis());
    match result {
        Ok(phase) => {
            report(Some(protocol), phase, "ok", elapsed);
            0
        }
        Err(Failure { phase, reason }) => {
            report(Some(protocol), phase, reason, elapsed);
            1
        }
    }
}

fn report(protocol: Option<&str>, phase: &str, reason: &str, elapsed_ms: Option<u128>) {
    println!(
        "{}",
        json!({
            "schema": 1, "operation": "probe", "ok": reason == "ok",
            "protocol": protocol, "phase": phase, "reason": reason, "elapsed_ms": elapsed_ms,
        })
    );
}

pub(crate) enum Connection {
    V4(rumqttc::AsyncClient, Box<rumqttc::EventLoop>),
    V5(v5::AsyncClient, Box<v5::EventLoop>),
}

pub(crate) enum Frame {
    Connected,
    SubscribeSent(u16),
    SubscribeAck(u16, bool),
    Echo,
    Disconnected,
    Other,
}

pub(crate) enum Options {
    V4(Box<rumqttc::MqttOptions>),
    V5(Box<v5::MqttOptions>),
}

impl Options {
    pub(crate) fn new(cli: &Cli, password: Option<String>) -> Result<Self, ()> {
        match cli.protocol_version {
            ProtocolVersion::Mqtt311 => crate::session::configure_options(cli, password)
                .map(Box::new)
                .map(Self::V4)
                .map_err(|_| ()),
            ProtocolVersion::Mqtt5 => {
                crate::v5_session::configure_options(cli, password.as_deref())
                    .map(Box::new)
                    .map(Self::V5)
                    .map_err(|_| ())
            }
        }
    }

    pub(crate) fn connection(&self, id: &str) -> Connection {
        match self {
            Self::V4(options) => {
                let mut options = options.as_ref().clone();
                options.set_client_id(id.to_owned());
                let (client, events) = rumqttc::AsyncClient::new(options, 8);
                Connection::V4(client, Box::new(events))
            }
            Self::V5(options) => {
                let mut options = options.as_ref().clone();
                options.set_client_id(id.to_owned());
                let (client, events) = v5::AsyncClient::new(options, 8);
                Connection::V5(client, Box::new(events))
            }
        }
    }
}

impl Connection {
    fn new(cli: &Cli, password: Option<String>) -> Result<Self, ()> {
        Ok(Options::new(cli, password)?.connection(&cli.client_id))
    }

    async fn diagnose(&mut self, probe: &Probe, deadline: Instant) -> Diagnosis<&'static str> {
        self.wait_connect(probe, deadline).await?;
        let phase = if let Some(topic) = &probe.topic {
            self.subscribe(probe, topic, deadline).await?;
            if let Some(marker) = &probe.marker {
                self.send_echo(topic, marker)?;
                while !matches!(self.next(probe, deadline, ROUND_TRIP).await?, Frame::Echo) {}
                ROUND_TRIP
            } else {
                SUBSCRIBE
            }
        } else {
            CONNECT
        };
        let queued = match self {
            Self::V4(client, _) => client.try_disconnect().is_ok(),
            Self::V5(client, _) => client.try_disconnect().is_ok(),
        };
        if queued {
            let close_deadline = deadline.min(Instant::now() + Duration::from_millis(100));
            while let Ok(frame) = self.next(probe, close_deadline, "disconnect").await {
                if matches!(frame, Frame::Disconnected) {
                    break;
                }
            }
        }
        Ok(phase)
    }

    pub(crate) async fn wait_connect(&mut self, probe: &Probe, deadline: Instant) -> Diagnosis<()> {
        while !matches!(self.next(probe, deadline, CONNECT).await?, Frame::Connected) {}
        Ok(())
    }

    pub(crate) async fn subscribe(
        &mut self,
        probe: &Probe,
        topic: &str,
        deadline: Instant,
    ) -> Diagnosis<()> {
        let queued = match self {
            Self::V4(client, _) => client.try_subscribe(topic, QoS::AtMostOnce).is_ok(),
            Self::V5(client, _) => client
                .try_subscribe(topic, v5::mqttbytes::QoS::AtMostOnce)
                .is_ok(),
        };
        if !queued {
            return Err(Failure {
                phase: SUBSCRIBE,
                reason: "request_failed",
            });
        }
        let mut expected = None;
        loop {
            match self.next(probe, deadline, SUBSCRIBE).await? {
                Frame::SubscribeSent(id) => expected = Some(id),
                Frame::SubscribeAck(id, true) if Some(id) == expected => return Ok(()),
                Frame::SubscribeAck(id, _) => {
                    return Err(Failure {
                        phase: SUBSCRIBE,
                        reason: if Some(id) == expected {
                            "rejected"
                        } else {
                            "unexpected_ack"
                        },
                    });
                }
                _ => {}
            }
        }
    }

    pub(crate) fn send_echo(&self, topic: &str, marker: &[u8]) -> Diagnosis<()> {
        let queued = match self {
            Self::V4(client, _) => client
                .try_publish(topic, QoS::AtMostOnce, false, marker)
                .is_ok(),
            Self::V5(client, _) => client
                .try_publish(
                    topic,
                    v5::mqttbytes::QoS::AtMostOnce,
                    false,
                    marker.to_vec(),
                )
                .is_ok(),
        };
        if queued {
            Ok(())
        } else {
            Err(Failure {
                phase: ROUND_TRIP,
                reason: "request_failed",
            })
        }
    }

    pub(crate) async fn next(
        &mut self,
        probe: &Probe,
        deadline: Instant,
        phase: &'static str,
    ) -> Diagnosis<Frame> {
        // An already ready socket cannot keep a probe alive past its deadline.
        if Instant::now() >= deadline {
            return Err(Failure {
                phase,
                reason: "timeout",
            });
        }
        let received = timeout_at(deadline, async {
            match self {
                Self::V4(_, events) => events
                    .poll()
                    .await
                    .map_err(v4_reason)
                    .and_then(|event| v4_frame(event, probe)),
                Self::V5(_, events) => events
                    .poll()
                    .await
                    .map_err(v5_reason)
                    .and_then(|event| v5_frame(event, probe)),
            }
        })
        .await
        .map_err(|_| Failure {
            phase,
            reason: "timeout",
        })?;
        received.map_err(|reason| Failure { phase, reason })
    }
}

fn echo(probe: &Probe, topic: &[u8], payload: &[u8], retain: bool) -> Frame {
    if !retain
        && probe.topic.as_deref().map(str::as_bytes) == Some(topic)
        && probe.marker.as_deref() == Some(payload)
    {
        Frame::Echo
    } else {
        Frame::Other
    }
}

fn v4_frame(event: rumqttc::Event, probe: &Probe) -> Result<Frame, &'static str> {
    use rumqttc::{Event, Incoming as Packet};
    Ok(match event {
        Event::Incoming(Packet::ConnAck(ack)) if !ack.session_present => Frame::Connected,
        Event::Incoming(Packet::ConnAck(_)) => return Err("unexpected_session"),
        Event::Outgoing(ClientPacket::Subscribe(id)) => Frame::SubscribeSent(id),
        Event::Incoming(Packet::SubAck(ack)) => Frame::SubscribeAck(
            ack.pkid,
            ack.return_codes == [v4::SubscribeReasonCode::Success(QoS::AtMostOnce)],
        ),
        Event::Incoming(Packet::Publish(msg)) => {
            echo(probe, msg.topic.as_bytes(), &msg.payload, msg.retain)
        }
        Event::Outgoing(ClientPacket::Disconnect) => Frame::Disconnected,
        _ => Frame::Other,
    })
}

fn v5_frame(event: v5::Event, probe: &Probe) -> Result<Frame, &'static str> {
    use v5::{
        Event, Incoming as Packet,
        mqttbytes::{QoS, v5::SubscribeReasonCode},
    };
    Ok(match event {
        Event::Incoming(Packet::ConnAck(ack)) => {
            crate::v5_session::validate_connack(&probe.cli, &ack).map_err(|_| "invalid_connack")?;
            Frame::Connected
        }
        Event::Outgoing(ClientPacket::Subscribe(id)) => Frame::SubscribeSent(id),
        Event::Incoming(Packet::SubAck(ack)) => Frame::SubscribeAck(
            ack.pkid,
            ack.return_codes == [SubscribeReasonCode::Success(QoS::AtMostOnce)],
        ),
        Event::Incoming(Packet::Publish(msg)) => echo(probe, &msg.topic, &msg.payload, msg.retain),
        Event::Incoming(Packet::Disconnect(_)) => return Err("broker_disconnected"),
        Event::Outgoing(ClientPacket::Disconnect) => Frame::Disconnected,
        _ => Frame::Other,
    })
}

fn io_reason(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::ConnectionRefused => "tcp_refused",
        io::ErrorKind::TimedOut => "timeout",
        _ => "transport_io",
    }
}

fn v4_reason(error: rumqttc::ConnectionError) -> &'static str {
    use rumqttc::ConnectionError as Error;
    use v4::ConnectReturnCode::{BadUserNamePassword, NotAuthorized};
    match error {
        Error::ConnectionRefused(BadUserNamePassword) => "bad_credentials",
        Error::ConnectionRefused(NotAuthorized) => "not_authorized",
        Error::ConnectionRefused(_) => "connack_refused",
        Error::Tls(_) => "tls_failed",
        Error::WsConnect(_) | Error::Websocket(_) | Error::ResponseValidation(_) => {
            "websocket_failed"
        }
        Error::Io(error) => io_reason(&error),
        Error::NetworkTimeout | Error::FlushTimeout => "timeout",
        _ => "protocol_error",
    }
}

fn v5_reason(error: v5::ConnectionError) -> &'static str {
    use v5::{
        ConnectionError as Error,
        mqttbytes::v5::ConnectReturnCode::{BadUserNamePassword, NotAuthorized},
    };
    match error {
        Error::ConnectionRefused(BadUserNamePassword) => "bad_credentials",
        Error::ConnectionRefused(NotAuthorized) => "not_authorized",
        Error::ConnectionRefused(_) => "connack_refused",
        Error::Tls(_) => "tls_failed",
        Error::WsConnect(_) | Error::Websocket(_) | Error::ResponseValidation(_) => {
            "websocket_failed"
        }
        Error::Io(error) => io_reason(&error),
        Error::Timeout(_) => "timeout",
        Error::MqttState(v5::StateError::ServerDisconnect { .. }) => "broker_disconnected",
        _ => "protocol_error",
    }
}

#[cfg(test)]
mod tests {
    use super::{Frame, echo, parse};

    #[test]
    fn round_trip_requires_exact_topic_fresh_marker_and_no_retain() {
        let probe = parse(
            ["--open-lab", "--topic", "fixture/topic", "--round-trip"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        let marker = probe.marker.as_deref().unwrap();
        assert!(matches!(
            echo(&probe, b"fixture/topic", marker, false),
            Frame::Echo
        ));
        for (topic, payload, retain) in [
            (b"wrong/topic".as_slice(), marker, false),
            (
                b"fixture/topic".as_slice(),
                b"wrong marker".as_slice(),
                false,
            ),
            (b"fixture/topic".as_slice(), marker, true),
        ] {
            assert!(matches!(echo(&probe, topic, payload, retain), Frame::Other));
        }
    }
}
