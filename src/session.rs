use std::{error::Error, io, time::Duration};

use rumqttc::{
    AsyncClient, Event, EventLoop, Incoming, MqttOptions, Outgoing, QoS, Transport,
    mqttbytes::v4::{ConnectReturnCode, Publish, SubscribeReasonCode},
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
    await_connack(&mut events).await?;

    let requested_qos = cli.qos;
    match cli.mode {
        Mode::Pub { topic, message } => {
            client
                .publish(topic, requested_qos, cli.retain, message.into_bytes())
                .await?;
            await_publish_complete(&mut events, requested_qos).await?;
            graceful_disconnect(&client, &mut events).await?;
            if requested_qos == QoS::AtMostOnce {
                println!("PUBLISH QoS 0 enviado; o protocolo nao confirma entrega ao assinante.");
            } else {
                println!(
                    "Publicacao confirmada pelo broker em QoS {}.",
                    qos_number(requested_qos)
                );
            }
        }
        Mode::Sub { filter, count } => {
            client.subscribe(filter, requested_qos).await?;
            let mut received = await_suback(&mut events, requested_qos).await?;
            println!("Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.");
            while count.is_none_or(|limit| received < limit) {
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => {
                        signal?;
                        break;
                    }
                    event = timeout(IDLE_TIMEOUT, events.poll()) => {
                        let event = event.map_err(|_| timed_out("resposta do broker"))??;
                        if let Event::Incoming(Incoming::Publish(publish)) = event {
                            print_publish(&publish);
                            received += 1;
                        }
                    }
                }
            }
            graceful_disconnect(&client, &mut events).await?;
        }
    }
    Ok(())
}

async fn await_connack(events: &mut EventLoop) -> AppResult<()> {
    let deadline = Instant::now() + INITIAL_TIMEOUT;
    loop {
        match next_before(events, deadline, "CONNACK").await? {
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
        match next_before(events, deadline, "envio de PUBLISH").await? {
            Event::Outgoing(Outgoing::Publish(0)) if qos == QoS::AtMostOnce => return Ok(()),
            Event::Incoming(Incoming::PubAck(_)) if qos == QoS::AtLeastOnce => return Ok(()),
            Event::Incoming(Incoming::PubComp(_)) if qos == QoS::ExactlyOnce => return Ok(()),
            Event::Outgoing(_) => {}
            Event::Incoming(Incoming::PubRec(_)) if qos == QoS::ExactlyOnce => {}
            Event::Incoming(_) => return Err(invalid_data("resposta MQTT inesperada").into()),
        }
    }
}

async fn await_suback(events: &mut EventLoop, qos: QoS) -> AppResult<u64> {
    let deadline = Instant::now() + INITIAL_TIMEOUT;
    let mut received = 0;
    loop {
        match next_before(events, deadline, "SUBACK").await? {
            Event::Incoming(Incoming::SubAck(ack)) => {
                if matches!(
                    ack.return_codes.as_slice(),
                    [SubscribeReasonCode::Success(granted)] if *granted == qos
                ) {
                    return Ok(received);
                }
                return Err(invalid_data("SUBACK recusou ou alterou QoS solicitado").into());
            }
            // MQTT 3.1.1 allows a matching publication before SUBACK.
            Event::Incoming(Incoming::Publish(publish)) => {
                print_publish(&publish);
                received += 1;
            }
            Event::Outgoing(_) => {}
            Event::Incoming(_) => {
                return Err(invalid_data("pacote inesperado antes de SUBACK").into());
            }
        }
    }
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

fn print_publish(publish: &Publish) {
    // Debug formatting escapes terminal control characters in untrusted fields.
    println!(
        "topico={:?} qos={} retain={} mensagem={:?}",
        publish.topic,
        qos_number(publish.qos),
        publish.retain,
        String::from_utf8_lossy(&publish.payload)
    );
}

const fn qos_number(qos: QoS) -> u8 {
    match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    }
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
