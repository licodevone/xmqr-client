//! Initial MQTT 5 pub/sub path. No retransmission after an uncertain send.
use std::{collections::HashMap, error::Error, io, time::Duration};

use rumqttc::v5::mqttbytes::v5::{
    ConnectReturnCode, Filter, LastWill, LastWillProperties, PubAckReason, PubCompReason,
    PubRecReason, Publish, PublishProperties, RetainForwardRule, SubscribeReasonCode,
    UnsubAckReason,
};
use rumqttc::{
    Outgoing, QoS, Transport,
    v5::{AsyncClient, ConnectionError, Event, Incoming, MqttOptions},
};
use tokio::time::{Instant, timeout, timeout_at};

use crate::{
    cli::{Cli, Mode, TransportMode},
    payload::Publication,
    tls,
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

fn protocol_error(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn connack_refusal(reason: ConnectReturnCode) -> io::Error {
    let code = match reason {
        ConnectReturnCode::Success => 0x00,
        ConnectReturnCode::RefusedProtocolVersion => 0x01,
        ConnectReturnCode::BadClientId => 0x02,
        ConnectReturnCode::ServiceUnavailable => 0x03,
        ConnectReturnCode::UnspecifiedError => 0x80,
        ConnectReturnCode::MalformedPacket => 0x81,
        ConnectReturnCode::ProtocolError => 0x82,
        ConnectReturnCode::ImplementationSpecificError => 0x83,
        ConnectReturnCode::UnsupportedProtocolVersion => 0x84,
        ConnectReturnCode::ClientIdentifierNotValid => 0x85,
        ConnectReturnCode::BadUserNamePassword => 0x86,
        ConnectReturnCode::NotAuthorized => 0x87,
        ConnectReturnCode::ServerUnavailable => 0x88,
        ConnectReturnCode::ServerBusy => 0x89,
        ConnectReturnCode::Banned => 0x8a,
        ConnectReturnCode::BadAuthenticationMethod => 0x8c,
        ConnectReturnCode::TopicNameInvalid => 0x90,
        ConnectReturnCode::PacketTooLarge => 0x95,
        ConnectReturnCode::QuotaExceeded => 0x97,
        ConnectReturnCode::PayloadFormatInvalid => 0x99,
        ConnectReturnCode::RetainNotSupported => 0x9a,
        ConnectReturnCode::QoSNotSupported => 0x9b,
        ConnectReturnCode::UseAnotherServer => 0x9c,
        ConnectReturnCode::ServerMoved => 0x9d,
        ConnectReturnCode::ConnectionRateExceeded => 0x9f,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("broker recusou CONNECT MQTT 5: {reason:?} (0x{code:02X})"),
    )
}

fn puback_refusal(reason: PubAckReason) -> io::Error {
    let code = match reason {
        PubAckReason::Success => 0x00,
        PubAckReason::NoMatchingSubscribers => 0x10,
        PubAckReason::UnspecifiedError => 0x80,
        PubAckReason::ImplementationSpecificError => 0x83,
        PubAckReason::NotAuthorized => 0x87,
        PubAckReason::TopicNameInvalid => 0x90,
        PubAckReason::PacketIdentifierInUse => 0x91,
        PubAckReason::QuotaExceeded => 0x97,
        PubAckReason::PayloadFormatInvalid => 0x99,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("broker recusou PUBLISH MQTT 5: {reason:?} (0x{code:02X})"),
    )
}

fn pubrec_refusal(reason: PubRecReason) -> io::Error {
    let code = match reason {
        PubRecReason::Success => 0x00,
        PubRecReason::NoMatchingSubscribers => 0x10,
        PubRecReason::UnspecifiedError => 0x80,
        PubRecReason::ImplementationSpecificError => 0x83,
        PubRecReason::NotAuthorized => 0x87,
        PubRecReason::TopicNameInvalid => 0x90,
        PubRecReason::PacketIdentifierInUse => 0x91,
        PubRecReason::QuotaExceeded => 0x97,
        PubRecReason::PayloadFormatInvalid => 0x99,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("broker recusou PUBLISH MQTT 5 em PUBREC: {reason:?} (0x{code:02X})"),
    )
}

fn pubcomp_refusal(reason: PubCompReason) -> io::Error {
    let code = match reason {
        PubCompReason::Success => 0x00,
        PubCompReason::PacketIdentifierNotFound => 0x92,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("broker recusou PUBLISH MQTT 5 em PUBCOMP: {reason:?} (0x{code:02X})"),
    )
}

fn suback_refusal(index: usize, reason: SubscribeReasonCode) -> io::Error {
    let code = match reason {
        SubscribeReasonCode::Success(qos) => qos as u8,
        SubscribeReasonCode::Failure | SubscribeReasonCode::Unspecified => 0x80,
        SubscribeReasonCode::ImplementationSpecific => 0x83,
        SubscribeReasonCode::NotAuthorized => 0x87,
        SubscribeReasonCode::TopicFilterInvalid => 0x8f,
        SubscribeReasonCode::PkidInUse => 0x91,
        SubscribeReasonCode::QuotaExceeded => 0x97,
        SubscribeReasonCode::SharedSubscriptionsNotSupported => 0x9e,
        SubscribeReasonCode::SubscriptionIdNotSupported => 0xa1,
        SubscribeReasonCode::WildcardSubscriptionsNotSupported => 0xa2,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "SUBACK MQTT 5 {} filtro {}: {reason:?} (0x{code:02X})",
            if matches!(reason, SubscribeReasonCode::Success(_)) {
                "alterou QoS do"
            } else {
                "recusou"
            },
            index + 1
        ),
    )
}

fn unsuback_refusal(index: usize, reason: UnsubAckReason) -> io::Error {
    let code = match reason {
        UnsubAckReason::Success => 0x00,
        UnsubAckReason::NoSubscriptionExisted => 0x11,
        UnsubAckReason::UnspecifiedError => 0x80,
        UnsubAckReason::ImplementationSpecificError => 0x83,
        UnsubAckReason::NotAuthorized => 0x87,
        UnsubAckReason::TopicFilterInvalid => 0x8f,
        UnsubAckReason::PacketIdentifierInUse => 0x91,
    };
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "UNSUBACK MQTT 5 recusou filtro {}: {reason:?} (0x{code:02X})",
            index + 1
        ),
    )
}

async fn unsubscribe_after_count(
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    filters: &[String],
    limit: Duration,
) -> AppResult<()> {
    for (index, filter) in filters.iter().enumerate() {
        client.try_unsubscribe(filter.clone())?;
        let deadline = Instant::now() + limit;
        loop {
            let event = timeout_at(deadline, events.poll())
                .await
                .map_err(|_| protocol_error("tempo esgotado aguardando UNSUBACK MQTT 5"))??;
            match event {
                Event::Incoming(Incoming::UnsubAck(ack)) => {
                    let [reason] = ack.reasons.as_slice() else {
                        return Err(protocol_error(
                            "UNSUBACK MQTT 5 retornou quantidade incorreta de motivos",
                        )
                        .into());
                    };
                    if *reason != UnsubAckReason::Success {
                        return Err(unsuback_refusal(index, *reason).into());
                    }
                    break;
                }
                Event::Incoming(
                    Incoming::Publish(_) | Incoming::PubRel(_) | Incoming::PingResp(_),
                )
                | Event::Outgoing(_) => {}
                Event::Incoming(_) => {
                    return Err(protocol_error(
                        "resposta MQTT 5 inesperada ao cancelar assinatura",
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
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

fn configure_connect_properties(options: &mut MqttOptions, cli: &Cli) {
    if let Some(value) = cli.request_problem_info {
        options.set_request_problem_info(Some(value));
    }
    if let Some(value) = cli.request_response_info {
        options.set_request_response_info(Some(value));
    }
    if !cli.connect_user_properties.is_empty() {
        options.set_user_properties(cli.connect_user_properties.clone());
    }
}

fn configure_options(cli: &Cli, password: Option<String>) -> AppResult<MqttOptions> {
    let mut options = MqttOptions::new(&cli.client_id, &cli.host, cli.port);
    options.set_clean_start(cli.clean_start == super::cli::SessionStart::Clean);
    if cli.session_expiry != 0 {
        options.set_session_expiry_interval(Some(cli.session_expiry));
    }
    options.set_keep_alive(Duration::from_secs(u64::from(cli.keepalive_seconds)));
    configure_connect_properties(&mut options, cli);
    if let Some(maximum) = cli.receive_maximum {
        options.set_receive_maximum(Some(maximum));
    }
    if let Some(maximum) = cli.maximum_packet_size {
        options.set_max_packet_size(Some(maximum));
    }
    if let Some(maximum) = cli.topic_alias_maximum {
        options.set_topic_alias_max(Some(maximum));
    }
    if let Some(will) = &cli.will {
        let qos = match will.qos {
            QoS::AtMostOnce => rumqttc::v5::mqttbytes::QoS::AtMostOnce,
            QoS::AtLeastOnce => rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
            QoS::ExactlyOnce => rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
        };
        let properties = will.delay_seconds.map(|seconds| LastWillProperties {
            delay_interval: Some(seconds),
            payload_format_indicator: None,
            message_expiry_interval: None,
            content_type: None,
            response_topic: None,
            correlation_data: None,
            user_properties: Vec::new(),
        });
        options.set_last_will(LastWill::new(
            will.topic.clone(),
            will.message.clone(),
            qos,
            will.retain,
            properties,
        ));
    }
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
    Ok(options)
}

async fn send_publication(
    client: &AsyncClient,
    cli: &Cli,
    topic: &str,
    payload: Vec<u8>,
    topic_alias: Option<u16>,
) -> AppResult<()> {
    let qos = match cli.qos {
        QoS::AtMostOnce => rumqttc::v5::mqttbytes::QoS::AtMostOnce,
        QoS::AtLeastOnce => rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
        QoS::ExactlyOnce => rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
    };
    if topic_alias.is_some()
        || cli.message_expiry.is_some()
        || cli.payload_format.is_some()
        || cli.content_type.is_some()
        || cli.response_topic.is_some()
        || cli.correlation_data.is_some()
        || !cli.publish_user_properties.is_empty()
    {
        client
            .publish_with_properties(
                topic,
                qos,
                cli.retain,
                payload,
                PublishProperties {
                    message_expiry_interval: cli.message_expiry,
                    payload_format_indicator: cli.payload_format,
                    content_type: cli.content_type.clone(),
                    response_topic: cli.response_topic.clone(),
                    correlation_data: cli.correlation_data.clone().map(Into::into),
                    user_properties: cli.publish_user_properties.clone(),
                    topic_alias,
                    ..PublishProperties::default()
                },
            )
            .await?;
    } else {
        client.publish(topic, qos, cli.retain, payload).await?;
    }
    Ok(())
}

async fn run_inner(
    cli: &Cli,
    password: Option<String>,
    publication: Option<Publication>,
) -> AppResult<()> {
    let options = configure_options(cli, password)?;
    let (client, mut events) = AsyncClient::new(options, 8);
    let limit = Duration::from_secs(u64::from(cli.timeout_seconds));
    let broker_topic_alias_maximum = loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado aguardando CONNACK MQTT 5"))?
            .map_err(|error| match error {
                ConnectionError::ConnectionRefused(reason) => connack_refusal(reason),
                _ => protocol_error("falha na conexao MQTT 5"),
            })?;
        match event {
            Event::Incoming(Incoming::ConnAck(ack)) if ack.code == ConnectReturnCode::Success => {
                if ack.session_present && cli.clean_start == super::cli::SessionStart::Clean {
                    return Err(protocol_error(
                        "CONNACK MQTT 5 informou sessao anterior apos Clean Start",
                    )
                    .into());
                }
                break ack
                    .properties
                    .as_ref()
                    .and_then(|properties| properties.topic_alias_max)
                    .unwrap_or(0);
            }
            Event::Incoming(Incoming::ConnAck(ack)) => {
                return Err(connack_refusal(ack.code).into());
            }
            Event::Outgoing(_) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta antes de CONNACK MQTT 5").into());
            }
        }
    };
    let Mode::Pub { topic, .. } = &cli.mode else {
        return subscribe(cli, &client, &mut events, limit).await;
    };
    if let Some(alias) = cli.publish_topic_alias
        && alias > broker_topic_alias_maximum
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("broker anuncia Topic Alias Maximum {broker_topic_alias_maximum}; alias solicitado {alias}"),
        )
        .into());
    }
    let publication = publication.ok_or_else(|| protocol_error("payload ausente"))?;
    for index in 0..cli.repeat {
        if index > 0 {
            wait_repeat_interval(&mut events, cli.repeat_interval_ms.unwrap()).await?;
        }
        let wire_topic = if index > 0 && cli.publish_topic_alias.is_some() {
            ""
        } else {
            topic
        };
        send_publication(
            &client,
            cli,
            wire_topic,
            publication.first.clone(),
            cli.publish_topic_alias,
        )
        .await?;
        await_publish_complete(&mut events, cli.qos, limit).await?;
        cli.output.complete(cli.qos);
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
    Ok(())
}

async fn wait_repeat_interval(
    events: &mut rumqttc::v5::EventLoop,
    milliseconds: u32,
) -> AppResult<()> {
    let deadline = Instant::now() + Duration::from_millis(u64::from(milliseconds));
    loop {
        tokio::select! {
            biased;
            () = tokio::time::sleep_until(deadline) => return Ok(()),
            event = events.poll() => match event {
                Ok(Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_))) => {},
                Ok(_) => return Err(protocol_error("resposta MQTT 5 inesperada entre publicacoes; lote parcial, nao reenviado").into()),
                Err(_) => return Err(protocol_error("conexao MQTT 5 interrompida entre publicacoes; lote parcial, nao reenviado").into()),
            },
        }
    }
}

async fn await_publish_complete(
    events: &mut rumqttc::v5::EventLoop,
    qos: QoS,
    limit: Duration,
) -> AppResult<()> {
    let deadline = Instant::now() + limit;
    loop {
        let event = timeout_at(deadline, events.poll())
            .await
            .map_err(|_| {
                protocol_error("publicacao MQTT 5 sem confirmacao; resultado desconhecido")
            })?
            .map_err(|_| {
                protocol_error("publicacao MQTT 5 interrompida; resultado desconhecido")
            })?;
        match event {
            Event::Outgoing(Outgoing::Publish(0)) if qos == QoS::AtMostOnce => return Ok(()),
            Event::Incoming(Incoming::PubAck(ack)) if qos == QoS::AtLeastOnce => {
                if ack.reason != PubAckReason::Success {
                    return Err(puback_refusal(ack.reason).into());
                }
                return Ok(());
            }
            Event::Incoming(Incoming::PubRec(ack)) if qos == QoS::ExactlyOnce => {
                if !matches!(
                    ack.reason,
                    PubRecReason::Success | PubRecReason::NoMatchingSubscribers
                ) {
                    return Err(pubrec_refusal(ack.reason).into());
                }
            }
            Event::Incoming(Incoming::PubComp(ack)) if qos == QoS::ExactlyOnce => {
                if ack.reason != PubCompReason::Success {
                    return Err(pubcomp_refusal(ack.reason).into());
                }
                return Ok(());
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => return Err(protocol_error("resposta MQTT 5 inesperada").into()),
        }
    }
}

fn resolved_incoming_topic(
    publish: &Publish,
    aliases: &mut HashMap<u16, String>,
    maximum: u16,
) -> AppResult<String> {
    let alias = publish
        .properties
        .as_ref()
        .and_then(|props| props.topic_alias);
    if alias.is_some_and(|value| value == 0 || value > maximum) {
        return Err(protocol_error("Topic Alias MQTT 5 fora do limite negociado").into());
    }
    if !publish.topic.is_empty() {
        let topic = std::str::from_utf8(&publish.topic)
            .map_err(|_| protocol_error("topico MQTT 5 invalido"))?;
        if topic.len() > 1024 {
            return Err(protocol_error("PUBLISH MQTT 5 excede limites XMQR").into());
        }
        if let Some(alias) = alias {
            aliases.insert(alias, topic.to_owned());
        }
        return Ok(topic.to_owned());
    }
    let alias = alias.ok_or_else(|| protocol_error("topico MQTT 5 vazio sem alias"))?;
    aliases
        .get(&alias)
        .cloned()
        .ok_or_else(|| protocol_error("Topic Alias MQTT 5 desconhecido").into())
}

fn subscription_filter(
    topic: String,
    qos: rumqttc::v5::mqttbytes::QoS,
    options: crate::cli::SubscriptionOptions,
) -> Filter {
    let mut request = Filter::new(topic, qos);
    request.nolocal = options.no_local;
    request.preserve_retain = options.retain_as_published;
    request.retain_forward_rule = match options.retain_handling {
        1 => RetainForwardRule::OnNewSubscribe,
        2 => RetainForwardRule::Never,
        _ => RetainForwardRule::OnEverySubscribe,
    };
    request
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
        QoS::ExactlyOnce => rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
    };
    client
        .subscribe_many(
            filters
                .iter()
                .cloned()
                .map(|filter| subscription_filter(filter, qos, cli.subscription_options)),
        )
        .await?;
    let mut acknowledged = false;
    let mut received = 0_u64;
    let mut aliases = HashMap::new();
    loop {
        if acknowledged && count.is_some_and(|target| received >= target) {
            unsubscribe_after_count(client, events, filters, limit).await?;
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
                if ack.return_codes.len() != filters.len() {
                    return Err(protocol_error(
                        "SUBACK MQTT 5 retornou quantidade incorreta de motivos",
                    )
                    .into());
                }
                for (index, reason) in ack.return_codes.iter().enumerate() {
                    if !matches!(reason, SubscribeReasonCode::Success(granted) if *granted == qos) {
                        return Err(suback_refusal(index, *reason).into());
                    }
                }
                acknowledged = true;
                cli.output
                    .status("Assinatura ativa (MQTT 5). Aguardando mensagens; Ctrl+C para sair.");
            }
            Event::Incoming(Incoming::Publish(publish)) => {
                let topic = resolved_incoming_topic(
                    &publish,
                    &mut aliases,
                    cli.topic_alias_maximum.unwrap_or(0),
                )?;
                if publish.payload.len() > 4096 {
                    return Err(protocol_error("PUBLISH MQTT 5 excede limites XMQR").into());
                }
                if cli.retained_filter.accepts(publish.retain)
                    && !exclusions
                        .iter()
                        .any(|filter| crate::cli::matches_filter(filter, &topic))
                    && count.is_none_or(|target| received < target)
                {
                    let qos = match publish.qos {
                        rumqttc::v5::mqttbytes::QoS::AtMostOnce => QoS::AtMostOnce,
                        rumqttc::v5::mqttbytes::QoS::AtLeastOnce => QoS::AtLeastOnce,
                        rumqttc::v5::mqttbytes::QoS::ExactlyOnce => QoS::ExactlyOnce,
                    };
                    let mut display =
                        rumqttc::mqttbytes::v4::Publish::new(&topic, qos, publish.payload.clone());
                    display.retain = publish.retain;
                    cli.output
                        .publish_mqtt5(&display, publish.properties.as_ref())?;
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
