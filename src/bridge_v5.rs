//! Engine opt-in MQTT 5: duas pernas independentes e transações locais.
use rumqttc::{
    Outgoing,
    v5::{
        self, Event, Incoming,
        mqttbytes::{
            QoS,
            v5::{
                ConnAck, ConnectReturnCode, Filter, PubAckReason, PubCompReason, PubRecReason,
                Publish, PublishProperties, RetainForwardRule, SubscribeReasonCode,
            },
        },
    },
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::time::{Instant, sleep, timeout};

use super::{
    AppResult, Config, Endpoint,
    bridge_metadata::{self, Settings},
    bridge_store::{Completion, Delivery, Phase, Store},
    invalid,
};

fn qos(value: u8) -> QoS {
    match value {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        _ => QoS::ExactlyOnce,
    }
}

fn options(endpoint: &Endpoint) -> AppResult<v5::MqttOptions> {
    let mut options = super::options_v5(endpoint, 2)?;
    options.set_receive_maximum(Some(1));
    Ok(options)
}

async fn connect(event_loop: &mut v5::EventLoop) -> Option<ConnAck> {
    match timeout(Duration::from_secs(8), event_loop.poll()).await {
        Ok(Ok(Event::Incoming(Incoming::ConnAck(packet)))) => Some(packet),
        _ => None,
    }
}

#[cfg(debug_assertions)]
async fn checkpoint(name: &str) {
    if std::env::var("XMQR_BRIDGE_TEST_CHECKPOINT").is_ok_and(|value| value == name) {
        eprintln!("Bridge: checkpoint {name}");
        sleep(Duration::from_secs(30)).await;
    }
}

#[cfg(not(debug_assertions))]
async fn checkpoint(_: &str) {}

fn session(packet: &ConnAck, known: bool, required: bool) -> AppResult<()> {
    if packet.code != ConnectReturnCode::Success
        || (packet.session_present && !known)
        || (!packet.session_present && required)
    {
        return Err(invalid(
            "sessao da bridge recusada, desconhecida ou ausente; estado preservado",
        )
        .into());
    }
    Ok(())
}

fn capacity(packet: &ConnAck, publish: &Publish) -> AppResult<()> {
    if let Some(properties) = &packet.properties
        && (properties
            .max_qos
            .is_some_and(|maximum| publish.qos as u8 > maximum)
            || (publish.retain && properties.retain_available == Some(0))
            || properties
                .max_packet_size
                .is_some_and(|limit| publish.size() > limit as usize))
    {
        return Err(invalid("destino nao permite QoS, retained ou tamanho solicitado").into());
    }
    Ok(())
}

async fn source_ack(store: &Store, source: &v5::AsyncClient) -> AppResult<()> {
    if let Some(delivery) = &store.document.pending
        && delivery.phase == Phase::Done
    {
        let item = &delivery.envelope;
        let mut packet = Publish::new(&item.source_topic, qos(item.source_qos), Vec::new(), None);
        packet.pkid = item.source_pkid;
        source.ack(&packet).await?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Discard {
    Expired,
    Path,
}

async fn finished(
    store: &mut Store,
    source: &v5::AsyncClient,
    discard: Option<Discard>,
) -> AppResult<()> {
    let mut next = store.document.clone();
    let delivery = next
        .pending
        .as_mut()
        .ok_or_else(|| invalid("ACK sem encaminhamento"))?;
    delivery.phase = Phase::Done;
    if let Some(reason) = discard {
        let count = match reason {
            Discard::Expired => &mut next.discarded_expired,
            Discard::Path => &mut next.discarded_path,
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| invalid("limite do contador de descartes; estado preservado"))?;
    }
    store.commit(next).await?;
    if discard.is_none() {
        checkpoint("after-destination-ack").await;
    }
    source_ack(store, source).await?;
    if store
        .document
        .pending
        .as_ref()
        .is_some_and(|item| item.envelope.source_qos == 0)
    {
        let mut next = store.document.clone();
        next.pending = None;
        store.commit(next).await?;
    }
    Ok(())
}

async fn refused(store: &mut Store) -> AppResult<()> {
    let mut next = store.document.clone();
    next.pending
        .as_mut()
        .ok_or_else(|| invalid("recusa sem encaminhamento pendente"))?
        .phase = Phase::Refused;
    store.commit(next).await?;
    Err(invalid("destino recusou encaminhamento; journal bloqueado e preservado").into())
}

fn transmission_properties(
    delivery: &Delivery,
    route_qos: u8,
    now: u64,
) -> AppResult<Option<PublishProperties>> {
    let mut properties = delivery.envelope.metadata.properties(now)?;
    if properties.is_none() && delivery.phase == Phase::Sent && route_qos > 0 {
        let mut metadata = delivery.envelope.metadata.clone();
        metadata.expires_ms = None;
        let mut remaining = metadata.properties(now)?.unwrap();
        remaining.message_expiry_interval = Some(0);
        properties = Some(remaining);
    }
    Ok(properties)
}

async fn transmit(
    config: &Config,
    store: &mut Store,
    source: &v5::AsyncClient,
    destination: &v5::AsyncClient,
    connack: &ConnAck,
) -> AppResult<()> {
    let Some(delivery) = store.document.pending.clone() else {
        return Ok(());
    };
    match delivery.phase {
        Phase::Refused => {
            return Err(invalid("encaminhamento recusado; journal preservado").into());
        }
        Phase::Done => {
            source_ack(store, source).await?;
            if delivery.envelope.source_qos == 0 {
                let mut next = store.document.clone();
                next.pending = None;
                store.commit(next).await?;
            }
        }
        Phase::Rel => destination.pubrel_packet(1).await?,
        Phase::Prepared | Phase::Sent => {
            let item = &delivery.envelope;
            let properties = transmission_properties(&delivery, config.route.qos, store.clock()?)?;
            if item.discard || properties.is_none() {
                eprintln!("Bridge: mensagem descartada por percurso ou expiracao");
                finished(
                    store,
                    source,
                    Some(if item.discard {
                        Discard::Path
                    } else {
                        Discard::Expired
                    }),
                )
                .await?;
                return Ok(());
            }
            let mut packet = Publish::new(
                &item.target_topic,
                qos(config.route.qos),
                item.payload.clone(),
                properties,
            );
            packet.retain = item.retain;
            packet.pkid = u16::from(config.route.qos != 0);
            packet.dup = delivery.phase == Phase::Sent && config.route.qos > 0;
            capacity(connack, &packet)?;
            let mut next = store.document.clone();
            next.pending.as_mut().unwrap().phase = Phase::Sent;
            store.commit(next).await?;
            checkpoint("before-destination-publish").await;
            // Descontar também espera de disco/fila local antes de novo envio.
            packet.properties =
                transmission_properties(&delivery, config.route.qos, store.clock()?)?;
            if packet.properties.is_none() {
                finished(store, source, Some(Discard::Expired)).await?;
                return Ok(());
            }
            capacity(connack, &packet)?;
            match config.route.qos {
                0 => {
                    destination
                        .publish_bytes_with_properties(
                            String::from_utf8(packet.topic.to_vec())?,
                            packet.qos,
                            packet.retain,
                            packet.payload,
                            packet.properties.unwrap(),
                        )
                        .await?;
                }
                1 => destination.publish_packet(packet).await?,
                _ => destination.publish_packet_manual_qos2(packet).await?,
            }
        }
    }
    Ok(())
}

async fn incoming(
    config: Arc<Config>,
    settings: Settings,
    message: Publish,
    store: &mut Store,
    source: &v5::AsyncClient,
    destination: &v5::AsyncClient,
    connack: &ConnAck,
) -> AppResult<()> {
    let now = store.clock()?;
    let duplicate = message.dup;
    let cfg = Arc::clone(&config);
    let envelope = tokio::task::spawn_blocking(move || {
        bridge_metadata::envelope(&cfg, &settings, &message, now)
    })
    .await??;
    if let Some(previous) = &store.document.pending {
        if envelope.source_qos > 0
            && envelope.source_pkid == previous.envelope.source_pkid
            && envelope.source_qos == previous.envelope.source_qos
            && duplicate
            && envelope.fingerprint == previous.envelope.fingerprint
        {
            source_ack(store, source).await?;
            return Ok(());
        }
        return Err(invalid("origem excedeu o limite de um encaminhamento pendente").into());
    }
    let mut next = store.document.clone();
    if envelope.source_qos == 2 {
        // Após PUBCOMP o PID pode nomear uma nova mensagem, mesmo com DUP=1.
        // A conclusão anterior serve apenas para PUBREL repetido (4.3.3-12).
        next.completed
            .retain(|item| item.pkid != envelope.source_pkid);
    }
    next.pending = Some(Delivery {
        envelope,
        phase: Phase::Prepared,
    });
    store.commit(next).await?;
    checkpoint("after-source-persist").await;
    transmit(&config, store, source, destination, connack).await
}

async fn pubrel(store: &mut Store, source: &v5::AsyncClient, pkid: u16) -> AppResult<()> {
    let mut next = store.document.clone();
    let known = if let Some(item) = &next.pending
        && item.envelope.source_qos == 2
        && item.envelope.source_pkid == pkid
        && item.phase == Phase::Done
    {
        next.completed.retain(|item| item.pkid != pkid);
        if next.completed.len() == 32 {
            next.completed.remove(0);
        }
        next.completed.push(Completion {
            pkid,
            fingerprint: item.envelope.fingerprint.clone(),
        });
        next.pending = None;
        store.commit(next).await?;
        checkpoint("before-source-pubcomp").await;
        true
    } else {
        next.completed.iter().any(|item| item.pkid == pkid)
    };
    source.try_pubcomp_packet(
        pkid,
        if known {
            PubCompReason::Success
        } else {
            PubCompReason::PacketIdentifierNotFound
        },
    )?;
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "transicoes MQTT das duas pernas permanecem explicitas"
)]
async fn drive(
    config: Arc<Config>,
    settings: &Settings,
    store: &mut Store,
    attempts: &mut u8,
) -> AppResult<()> {
    let cfg = Arc::clone(&config);
    let (source_options, destination_options) = tokio::task::spawn_blocking(move || {
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((
            options(&cfg.source)?,
            options(&cfg.destination)?,
        ))
    })
    .await??;
    let (source, mut source_loop) = v5::AsyncClient::new(source_options, 8);
    source_loop.state.set_manual_incoming_qos2(true);
    let (destination, mut destination_loop) = v5::AsyncClient::new(destination_options, 8);
    let Some(destination_connack) = connect(&mut destination_loop).await else {
        return Ok(());
    };
    let pending = store.document.pending.as_ref();
    let destination_required = pending
        .is_some_and(|item| config.route.qos > 0 && matches!(item.phase, Phase::Sent | Phase::Rel));
    let source_required = pending.is_some_and(|item| item.envelope.source_qos > 0);
    session(
        &destination_connack,
        store.document.destination_known,
        destination_required,
    )?;
    // Persistir cada aceite antes da próxima conexão, que pode falhar.
    if !store.document.destination_known {
        let mut next = store.document.clone();
        next.destination_known = true;
        store.commit(next).await?;
    }
    let Some(source_connack) = connect(&mut source_loop).await else {
        return Ok(());
    };
    session(
        &source_connack,
        store.document.source_known,
        source_required,
    )?;
    let mut next = store.document.clone();
    if !source_connack.session_present {
        next.completed.clear();
    }
    next.source_known = true;
    next.destination_known = true;
    store.commit(next).await?;
    let mut filter = Filter::new(
        format!("{}#", config.route.source_prefix),
        qos(config.route.qos),
    );
    filter.preserve_retain = true;
    filter.retain_forward_rule = RetainForwardRule::OnNewSubscribe;
    source.subscribe_many([filter]).await?;
    transmit(&config, store, &source, &destination, &destination_connack).await?;
    let mut progress = Instant::now();
    loop {
        tokio::select! {
            () = sleep(Duration::from_secs(1)) => {
                if store.document.pending.is_some() && progress.elapsed() > Duration::from_secs(20) {
                    return Ok(());
                }
            },
            event = source_loop.poll() => match event {
                Ok(Event::Incoming(Incoming::SubAck(packet))) => {
                    if !matches!(packet.return_codes.as_slice(), [SubscribeReasonCode::Success(actual)] if *actual == qos(config.route.qos)) {
                        return Err(invalid("assinatura da bridge recusada").into());
                    }
                    eprintln!("Bridge: origem assinada (rota {})", config.link_id.as_deref().unwrap_or("unica"));
                },
                Ok(Event::Incoming(Incoming::Publish(packet))) => {
                    incoming(Arc::clone(&config), settings.clone(), packet, store,
                             &source, &destination, &destination_connack).await?;
                    progress = Instant::now();
                },
                Ok(Event::Incoming(Incoming::PubRel(packet))) => {
                    pubrel(store, &source, packet.pkid).await?;
                    *attempts = 0;
                    progress = Instant::now();
                },
                Ok(Event::Outgoing(Outgoing::PubAck(pkid))) => {
                    if store.document.pending.as_ref().is_some_and(|item|
                        item.phase == Phase::Done && item.envelope.source_qos == 1 && item.envelope.source_pkid == pkid) {
                        let mut next = store.document.clone();
                        next.pending = None;
                        store.commit(next).await?;
                        *attempts = 0;
                    }
                },
                Ok(Event::Outgoing(Outgoing::PubRec(_))) => checkpoint("after-source-pubrec").await,
                Ok(Event::Incoming(Incoming::Disconnect(_))) => return Err(invalid("origem encerrou a sessao").into()),
                Err(_) => return Ok(()),
                Ok(_) => {},
            },
            event = destination_loop.poll() => match event {
                Ok(Event::Incoming(Incoming::PubAck(packet))) => {
                    if packet.pkid != 1 || config.route.qos != 1
                        || !store.document.pending.as_ref().is_some_and(|item| item.phase == Phase::Sent) {
                        return Err(invalid("PUBACK de destino recusado ou inesperado").into());
                    }
                    if !matches!(packet.reason, PubAckReason::Success | PubAckReason::NoMatchingSubscribers) {
                        return refused(store).await;
                    }
                    finished(store, &source, None).await?;
                    progress = Instant::now();
                },
                Ok(Event::Incoming(Incoming::PubRec(packet))) => {
                    if packet.pkid != 1 || config.route.qos != 2
                        || !store.document.pending.as_ref().is_some_and(|item| item.phase == Phase::Sent) {
                        return Err(invalid("PUBREC de destino recusado ou inesperado").into());
                    }
                    if !matches!(packet.reason, PubRecReason::Success | PubRecReason::NoMatchingSubscribers) {
                        return refused(store).await;
                    }
                    let mut next = store.document.clone();
                    next.pending.as_mut().unwrap().phase = Phase::Rel;
                    store.commit(next).await?;
                    checkpoint("before-destination-pubrel").await;
                    destination.pubrel_packet(1).await?;
                    progress = Instant::now();
                },
                Ok(Event::Incoming(Incoming::PubComp(packet))) => {
                    if packet.pkid != 1 || config.route.qos != 2
                        || !store.document.pending.as_ref().is_some_and(|item| item.phase == Phase::Rel) {
                        return Err(invalid("PUBCOMP de destino recusado ou inesperado").into());
                    }
                    if packet.reason != PubCompReason::Success {
                        return refused(store).await;
                    }
                    finished(store, &source, None).await?;
                    progress = Instant::now();
                },
                Ok(Event::Outgoing(Outgoing::Publish(0))) if config.route.qos == 0 => {
                    finished(store, &source, None).await?;
                    *attempts = 0;
                },
                Ok(Event::Incoming(Incoming::Disconnect(_))) => return Err(invalid("destino encerrou a sessao").into()),
                Err(_) => return Ok(()),
                Ok(_) => {},
            },
        }
    }
}

async fn run_loop(config: Config, directory: PathBuf, settings: Settings) -> AppResult<()> {
    let identity = serde_json::to_vec(&(
        (
            &config.source.host,
            config.source.port,
            &config.source.client_id,
            &config.source.username,
        ),
        (
            &config.destination.host,
            config.destination.port,
            &config.destination.client_id,
            &config.destination.username,
        ),
        (
            &config.route.source_prefix,
            &config.route.target_prefix,
            config.route.qos,
        ),
        (
            &settings.bridge_id,
            settings.max_hops,
            format!("{:?}", settings.response_policy),
        ),
    ))?;
    let mut store = Store::open(directory, identity).await?;
    if store
        .document
        .pending
        .as_ref()
        .is_some_and(|item| item.phase == Phase::Refused)
    {
        return Err(
            invalid("encaminhamento recusado exige intervencao; journal preservado").into(),
        );
    }
    let config = Arc::new(config);
    let mut attempts = 0;
    loop {
        Box::pin(drive(
            Arc::clone(&config),
            &settings,
            &mut store,
            &mut attempts,
        ))
        .await?;
        attempts += 1;
        if attempts > config.max_retries {
            return Err(invalid("bridge sem conexao apos tentativas; estado preservado").into());
        }
        eprintln!("Bridge: reconectando com estado duravel");
        sleep(Duration::from_secs(u64::from(attempts.min(5)))).await;
    }
}

pub async fn run(config: Config, directory: PathBuf, settings: Settings) -> AppResult<()> {
    // O ouvinte permanece registrado também durante awaits de conexão e disco.
    tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = Box::pin(run_loop(config, directory, settings)) => result,
    }
}
