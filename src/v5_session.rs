//! Initial MQTT 5 publish-only path. No retransmission after an uncertain send.
use std::{error::Error, io, time::Duration};

use rumqttc::v5::mqttbytes::v5::{ConnectReturnCode, Filter, PubAckReason, SubscribeReasonCode};
use rumqttc::{
    Outgoing, QoS, Transport,
    v5::{AsyncClient, ConnectionError, Event, Incoming, MqttOptions},
};
use tokio::time::timeout;

use crate::{
    cli::{Cli, Mode, TransportMode},
    payload::Publication,
    tls,
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

fn protocol_error(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub async fn run(
    cli: Cli,
    password: Option<String>,
    publication: Option<Publication>,
) -> AppResult<()> {
    let total = cli
        .total_timeout_seconds
        .map(|s| Duration::from_secs(u64::from(s)));
    let operation = async {
        if let Some(total) = total {
            timeout(total, run_inner(&cli, password, publication)).await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut,
                    "tempo total esgotado; publicacao pode ter resultado desconhecido, nao reenviada"))?
        } else {
            run_inner(&cli, password, publication).await
        }
    };
    tokio::select! {
        biased;
        signal = crate::signals::shutdown() => { signal?; Ok(()) },
        result = operation => result,
    }
}

async fn run_inner(
    cli: &Cli,
    password: Option<String>,
    publication: Option<Publication>,
) -> AppResult<()> {
    let mut options = MqttOptions::new(&cli.client_id, &cli.host, cli.port);
    options.set_clean_start(true);
    options.set_keep_alive(Duration::from_secs(u64::from(cli.keepalive_seconds)));
    if matches!(
        cli.transport_mode,
        TransportMode::Secure | TransportMode::TlsPassword
    ) {
        options.set_transport(Transport::tls_with_config(
            tls::build_client_config(cli)?.into(),
        ));
    }
    if let (Some(username), Some(password)) = (cli.username.clone(), password) {
        options.set_credentials(username, password);
    }
    // CONNECT stays property-free in the broker's initial MQTT 5 subset.
    let (client, mut events) = AsyncClient::new(options, 8);
    let limit = Duration::from_secs(u64::from(cli.timeout_seconds));
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado aguardando CONNACK MQTT 5"))?
            .map_err(|error| match error {
                ConnectionError::ConnectionRefused(_) => {
                    protocol_error("broker recusou CONNECT MQTT 5")
                }
                _ => protocol_error("falha na conexao MQTT 5"),
            })?;
        match event {
            Event::Incoming(Incoming::ConnAck(ack)) if ack.code == ConnectReturnCode::Success => {
                break;
            }
            Event::Incoming(Incoming::ConnAck(_)) => {
                return Err(protocol_error("broker recusou CONNECT MQTT 5").into());
            }
            Event::Outgoing(_) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta antes de CONNACK MQTT 5").into());
            }
        }
    }
    let Mode::Pub { topic, .. } = &cli.mode else {
        return subscribe(cli, &client, &mut events, limit).await;
    };
    let publication = publication.ok_or_else(|| protocol_error("payload ausente"))?;
    let v5_qos = match cli.qos {
        QoS::AtMostOnce => rumqttc::v5::mqttbytes::QoS::AtMostOnce,
        QoS::AtLeastOnce => rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
        QoS::ExactlyOnce => return Err(protocol_error("MQTT 5 QoS 2 indisponivel").into()),
    };
    client
        .publish(topic, v5_qos, cli.retain, publication.first)
        .await?;
    loop {
        let event = timeout(limit, events.poll()).await.map_err(|_| {
            protocol_error("publicacao MQTT 5 sem confirmacao; resultado desconhecido")
        })??;
        match event {
            Event::Outgoing(Outgoing::Publish(0)) if cli.qos == QoS::AtMostOnce => break,
            Event::Incoming(Incoming::PubAck(ack)) if cli.qos == QoS::AtLeastOnce => {
                if ack.reason != PubAckReason::Success {
                    return Err(protocol_error("broker recusou PUBLISH MQTT 5").into());
                }
                break;
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => return Err(protocol_error("resposta MQTT 5 inesperada").into()),
        }
    }
    client.try_disconnect()?;
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado enviando DISCONNECT MQTT 5"))??;
        match event {
            Event::Outgoing(Outgoing::Disconnect) => break,
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta MQTT 5 inesperada no encerramento").into());
            }
        }
    }
    cli.output.complete(cli.qos);
    Ok(())
}

async fn subscribe(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    limit: Duration,
) -> AppResult<()> {
    let Mode::Sub {
        filters,
        exclusions,
        count,
    } = &cli.mode
    else {
        return Err(protocol_error("modo de assinatura esperado").into());
    };
    let qos = match cli.qos {
        QoS::AtMostOnce => rumqttc::v5::mqttbytes::QoS::AtMostOnce,
        QoS::AtLeastOnce => rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
        QoS::ExactlyOnce => return Err(protocol_error("MQTT 5 QoS 2 indisponivel").into()),
    };
    client
        .subscribe_many(
            filters
                .iter()
                .cloned()
                .map(|filter| Filter::new(filter, qos)),
        )
        .await?;
    let mut acknowledged = false;
    let mut received = 0_u64;
    loop {
        if acknowledged && count.is_some_and(|target| received >= target) {
            client.try_disconnect()?;
            loop {
                let event = timeout(limit, events.poll())
                    .await
                    .map_err(|_| protocol_error("tempo esgotado enviando DISCONNECT MQTT 5"))??;
                if matches!(event, Event::Outgoing(Outgoing::Disconnect)) {
                    break;
                }
            }
            return Ok(());
        }
        let wait = if acknowledged {
            Duration::from_secs(90)
        } else {
            limit
        };
        let event = timeout(wait, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado aguardando SUBACK/mensagem MQTT 5"))??;
        match event {
            Event::Incoming(Incoming::SubAck(ack)) if !acknowledged => {
                if ack.return_codes.len() != filters.len()
                    || ack.return_codes.iter().any(|code| !matches!(code, SubscribeReasonCode::Success(granted) if *granted == qos)) {
                    return Err(protocol_error("SUBACK MQTT 5 recusou ou alterou QoS solicitado").into());
                }
                acknowledged = true;
                cli.output
                    .status("Assinatura ativa (MQTT 5). Aguardando mensagens; Ctrl+C para sair.");
            }
            Event::Incoming(Incoming::Publish(publish)) => {
                let topic = std::str::from_utf8(&publish.topic)
                    .map_err(|_| protocol_error("topico MQTT 5 invalido"))?;
                if publish.payload.len() > 4096 || topic.len() > 1024 {
                    return Err(protocol_error("PUBLISH MQTT 5 excede limites XMQR").into());
                }
                if cli.retained_filter.accepts(publish.retain)
                    && !exclusions
                        .iter()
                        .any(|filter| crate::cli::matches_filter(filter, topic))
                    && count.is_none_or(|target| received < target)
                {
                    let qos = match publish.qos {
                        rumqttc::v5::mqttbytes::QoS::AtMostOnce => QoS::AtMostOnce,
                        rumqttc::v5::mqttbytes::QoS::AtLeastOnce => QoS::AtLeastOnce,
                        rumqttc::v5::mqttbytes::QoS::ExactlyOnce => {
                            return Err(protocol_error("PUBLISH MQTT 5 QoS 2 indisponivel").into());
                        }
                    };
                    let mut display =
                        rumqttc::mqttbytes::v4::Publish::new(topic, qos, publish.payload.clone());
                    display.retain = publish.retain;
                    cli.output.publish(&display)?;
                    received += 1;
                }
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta MQTT 5 inesperada na assinatura").into());
            }
        }
    }
}
