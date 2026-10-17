//! Initial MQTT 5 pub/sub path. No retransmission after an uncertain send.
use std::{collections::HashMap, error::Error, io, time::Duration};

use rumqttc::v5::mqttbytes::v5::{
    Auth, AuthProperties, AuthReasonCode, ConnAck, ConnectReturnCode, Filter, LastWill,
    LastWillProperties, PubAckReason, PubCompReason, PubRecReason, Publish, PublishProperties,
    RetainForwardRule, SubscribeProperties, SubscribeReasonCode, UnsubAckReason,
    UnsubscribeProperties,
};
use rumqttc::{
    Outgoing, QoS, Transport,
    v5::{AsyncClient, ConnectionError, Event, Incoming, MqttOptions},
};
use tokio::time::{Instant, timeout, timeout_at};

use crate::{
    cli::{Cli, ClientIdMode, Mode, ReauthMode, TransportMode},
    payload::Publication,
    tls,
};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[cfg(test)]
mod assigned_client_id_tests {
    use super::{configure_options, validate_connack};
    use bytes::BytesMut;
    use rumqttc::v5::mqttbytes::v5::{ConnAck, Connect, Packet};

    fn cli(assigned: bool) -> crate::cli::Cli {
        let mut args = vec![
            "sub",
            "--open-lab",
            "--topic",
            "x",
            "--protocol-version",
            "5",
        ];
        if assigned {
            args.push("--server-assigned-client-id");
        }
        crate::cli::parse(args.into_iter().map(str::to_owned)).unwrap()
    }

    fn connack(wire: &[u8]) -> ConnAck {
        let mut bytes = BytesMut::from(wire);
        match Packet::read(&mut bytes, Some(65_536)).unwrap() {
            Packet::ConnAck(ack) => ack,
            _ => panic!("expected CONNACK"),
        }
    }

    #[test]
    fn empty_connect_id_requires_valid_assigned_connack_property() {
        let assigned = cli(true);
        assert_eq!(configure_options(&assigned, None).unwrap().client_id(), "");
        let id_packet = connack(b"\x20\x0b\x00\x00\x08\x12\x00\x05xmqr1");
        assert_eq!(
            validate_connack(&assigned, &id_packet).unwrap(),
            Some("xmqr1".into())
        );
        assert!(validate_connack(&assigned, &connack(b"\x20\x03\x00\x00\x00")).is_err());
        assert!(validate_connack(&cli(false), &id_packet).is_err());
        let invalid = connack(b"\x20\x07\x00\x00\x04\x12\x00\x01\n");
        assert!(validate_connack(&assigned, &invalid).is_err());
    }

    #[test]
    fn duplicate_assigned_identifier_is_malformed_on_wire() {
        let mut bytes = BytesMut::from(&b"\x20\x0b\x00\x00\x08\x12\x00\x01a\x12\x00\x01b"[..]);
        assert!(Packet::read(&mut bytes, Some(65_536)).is_err());
    }

    #[test]
    fn mqtt5_connect_encodes_reused_assigned_id_length() {
        let id = "xmqr-0123456789abcdef0123456789abcdef";
        let cli = crate::cli::parse(
            [
                "sub",
                "--open-lab",
                "--topic",
                "x",
                "--protocol-version",
                "5",
                "--client-id",
                id,
                "--clean-start",
                "false",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        let options = configure_options(&cli, None).unwrap();
        let mut wire = BytesMut::new();
        Connect {
            client_id: options.client_id(),
            keep_alive: u16::try_from(options.keep_alive().as_secs()).unwrap(),
            clean_start: options.clean_start(),
            properties: options.connect_properties(),
        }
        .write(&None, &None, &mut wire)
        .unwrap();
        assert_eq!(&wire[13..15], &[0, 37]);
        assert_eq!(&wire[15..], id.as_bytes());
    }

    #[test]
    fn mqtt5_empty_id_encodes_durable_session_expiry() {
        let cli = crate::cli::parse(
            [
                "pub",
                "--open-lab",
                "--topic",
                "x",
                "--message",
                "x",
                "--protocol-version",
                "5",
                "--server-assigned-client-id",
                "--session-expiry-seconds",
                "60",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        let options = configure_options(&cli, None).unwrap();
        let mut wire = BytesMut::new();
        Connect {
            client_id: options.client_id(),
            keep_alive: u16::try_from(options.keep_alive().as_secs()).unwrap(),
            clean_start: options.clean_start(),
            properties: options.connect_properties(),
        }
        .write(&None, &None, &mut wire)
        .unwrap();
        assert_eq!(&wire[12..], b"\x05\x11\x00\x00\x00\x3c\x00\x00");
    }
}

fn protocol_error(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn check_subscription_features(
    cli: &Cli,
    identifiers_available: Option<u8>,
    shared_available: Option<u8>,
) -> AppResult<()> {
    if cli.subscription_options.identifier.is_some() && identifiers_available == Some(0) {
        return Err(protocol_error("broker nao oferece Subscription Identifier MQTT 5").into());
    }
    if matches!(&cli.mode, Mode::Sub { filters, .. } if filters.iter().any(|filter| filter.starts_with("$share/")))
        && shared_available == Some(0)
    {
        return Err(protocol_error("broker nao oferece shared subscriptions MQTT 5").into());
    }
    Ok(())
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
    user_properties: &[(String, String)],
    limit: Duration,
) -> AppResult<()> {
    for (index, filter) in filters.iter().enumerate() {
        if user_properties.is_empty() {
            client.try_unsubscribe(filter.clone())?;
        } else {
            client.try_unsubscribe_with_properties(
                filter.clone(),
                UnsubscribeProperties {
                    user_properties: user_properties.to_vec(),
                },
            )?;
        }
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

fn configure_options(cli: &Cli, password: Option<&str>) -> AppResult<MqttOptions> {
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
        let has_properties = will.delay_seconds.is_some()
            || will.message_expiry_seconds.is_some()
            || will.payload_format_indicator.is_some()
            || will.content_type.is_some()
            || will.response_topic.is_some()
            || will.correlation_data.is_some()
            || !will.user_properties.is_empty();
        let properties = has_properties.then(|| LastWillProperties {
            delay_interval: will.delay_seconds,
            payload_format_indicator: will.payload_format_indicator,
            message_expiry_interval: will.message_expiry_seconds,
            content_type: will.content_type.clone(),
            response_topic: will.response_topic.clone(),
            correlation_data: will.correlation_data.clone().map(Into::into),
            user_properties: will.user_properties.clone(),
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
    configure_auth(&mut options, cli, password)?;
    Ok(options)
}

fn plain_auth_data(cli: &Cli, password: &str) -> AppResult<Vec<u8>> {
    let username = cli
        .username
        .as_ref()
        .ok_or_else(|| protocol_error("usuario ausente"))?;
    let mut data = Vec::with_capacity(username.len() + password.len() + 2);
    data.push(0);
    data.extend_from_slice(username.as_bytes());
    data.push(0);
    data.extend_from_slice(password.as_bytes());
    Ok(data)
}

fn configure_auth(options: &mut MqttOptions, cli: &Cli, password: Option<&str>) -> AppResult<()> {
    if cli.auth_method.as_deref() == Some("plain") {
        let password = password.ok_or_else(|| protocol_error("senha ausente"))?;
        options.set_authentication_method(Some("PLAIN".to_owned()));
        options.set_authentication_data(Some(plain_auth_data(cli, password)?.into()));
    } else if let (Some(username), Some(password)) = (cli.username.clone(), password) {
        options.set_credentials(username, password.to_owned());
    }
    Ok(())
}

fn confirm_auth_method(cli: &Cli, method: Option<&str>) -> AppResult<()> {
    if cli.auth_method.as_deref() == Some("plain") && method != Some("PLAIN") {
        return Err(protocol_error("CONNACK MQTT 5 nao confirmou o metodo PLAIN").into());
    }
    Ok(())
}

fn validate_connack(cli: &Cli, ack: &ConnAck) -> AppResult<Option<String>> {
    confirm_auth_method(
        cli,
        ack.properties
            .as_ref()
            .and_then(|p| p.authentication_method.as_deref()),
    )?;
    if ack.session_present && cli.clean_start == super::cli::SessionStart::Clean {
        return Err(
            protocol_error("CONNACK MQTT 5 informou sessao anterior apos Clean Start").into(),
        );
    }
    let assigned = ack
        .properties
        .as_ref()
        .and_then(|properties| properties.assigned_client_identifier.as_deref());
    if cli.client_id_mode == ClientIdMode::ServerAssigned {
        let id = assigned
            .ok_or_else(|| protocol_error("CONNACK MQTT 5 omitiu Client Identifier atribuido"))?;
        if id.is_empty()
            || id.len() > 256
            || id.chars().any(|ch| {
                let code = ch as u32;
                ch == '\0'
                    || ch.is_control()
                    || (0xfdd0..=0xfdef).contains(&code)
                    || code & 0xffff == 0xfffe
                    || code & 0xffff == 0xffff
            })
        {
            return Err(protocol_error("CONNACK MQTT 5 trouxe Client Identifier invalido").into());
        }
    } else if assigned.is_some() {
        return Err(protocol_error("CONNACK MQTT 5 atribuiu ID sem solicitação").into());
    }
    check_subscription_features(
        cli,
        ack.properties
            .as_ref()
            .and_then(|p| p.subscription_identifiers_available),
        ack.properties
            .as_ref()
            .and_then(|p| p.shared_subscription_available),
    )?;
    Ok(assigned.map(str::to_owned))
}

fn plain_reauth_request(cli: &Cli, password: &str) -> AppResult<Auth> {
    Ok(Auth {
        code: AuthReasonCode::ReAuthentivate,
        properties: Some(AuthProperties {
            method: Some("PLAIN".to_owned()),
            data: Some(plain_auth_data(cli, password)?.into()),
            ..AuthProperties::default()
        }),
    })
}

async fn reauthenticate(
    cli: &Cli,
    password: &str,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    limit: Duration,
) -> AppResult<()> {
    let request = plain_reauth_request(cli, password)?;
    client
        .auth(request)
        .await
        .map_err(|_| protocol_error("falha ao enviar AUTH MQTT 5"))?;
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado aguardando AUTH MQTT 5"))?
            .map_err(|_| protocol_error("falha aguardando AUTH MQTT 5"))?;
        match event {
            Event::Incoming(Incoming::Auth(auth))
                if auth.code == AuthReasonCode::Success
                    && auth.properties.as_ref().and_then(|p| p.method.as_deref())
                        == Some("PLAIN") =>
            {
                return Ok(());
            }
            Event::Outgoing(_) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("AUTH MQTT 5 nao confirmou PLAIN").into());
            }
        }
    }
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
    let options = configure_options(cli, password.as_deref())?;
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
                if let Some(id) = validate_connack(cli, &ack)? {
                    eprintln!("MQTT 5 Client Identifier atribuido: {id}");
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
    if cli.reauth_mode == ReauthMode::BeforeIo {
        let password = password
            .as_deref()
            .ok_or_else(|| protocol_error("senha ausente"))?;
        reauthenticate(cli, password, &client, &mut events, limit).await?;
    }
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

async fn send_subscribe(
    client: &AsyncClient,
    filters: &[String],
    qos: rumqttc::v5::mqttbytes::QoS,
    options: crate::cli::SubscriptionOptions,
    user_properties: &[(String, String)],
) -> AppResult<()> {
    let requests = filters
        .iter()
        .cloned()
        .map(|filter| subscription_filter(filter, qos, options));
    if options.identifier.is_some() || !user_properties.is_empty() {
        client
            .subscribe_many_with_properties(
                requests,
                SubscribeProperties {
                    id: options.identifier,
                    user_properties: user_properties.to_vec(),
                },
            )
            .await?;
    } else {
        client.subscribe_many(requests).await?;
    }
    Ok(())
}

fn validate_subscription_identifiers(publish: &Publish) -> AppResult<()> {
    if publish.properties.as_ref().is_some_and(|properties| {
        properties
            .subscription_identifiers
            .iter()
            .any(|id| !(1..=268_435_455).contains(id))
    }) {
        return Err(
            protocol_error("PUBLISH MQTT 5 contem Subscription Identifier invalido").into(),
        );
    }
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
        QoS::ExactlyOnce => rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
    };
    send_subscribe(
        client,
        filters,
        qos,
        cli.subscription_options,
        &cli.subscribe_user_properties,
    )
    .await?;
    let mut acknowledged = false;
    let mut received = 0_u64;
    let mut aliases = HashMap::new();
    loop {
        if acknowledged && count.is_some_and(|target| received >= target) {
            finish_counted_subscription(cli, client, events, filters, limit).await?;
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
                validate_subscription_identifiers(&publish)?;
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

async fn finish_counted_subscription(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    filters: &[String],
    limit: Duration,
) -> AppResult<()> {
    unsubscribe_after_count(
        client,
        events,
        filters,
        &cli.unsubscribe_user_properties,
        limit,
    )
    .await?;
    client.try_disconnect()?;
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado enviando DISCONNECT MQTT 5"))??;
        if matches!(event, Event::Outgoing(Outgoing::Disconnect)) {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod plain_auth_tests {
    use super::{MqttOptions, configure_auth, confirm_auth_method, plain_reauth_request};
    use bytes::BytesMut;
    use rumqttc::v5::mqttbytes::v5::Packet;

    #[test]
    fn plain_data_uses_auth_properties_without_legacy_credentials() {
        let args = [
            "sub",
            "--tls-password",
            "--topic",
            "p159/value",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--protocol-version",
            "5",
            "--auth-method",
            "plain",
        ];
        let cli = crate::cli::parse(args.into_iter().map(str::to_owned)).unwrap();
        let mut options = MqttOptions::new("test-client", "localhost", 8883);
        configure_auth(&mut options, &cli, Some("test-secret")).unwrap();
        assert_eq!(options.authentication_method().as_deref(), Some("PLAIN"));
        assert_eq!(
            options.authentication_data().as_deref(),
            Some(b"\0device\0test-secret".as_slice())
        );
        assert!(options.credentials().is_none());
        assert!(confirm_auth_method(&cli, Some("PLAIN")).is_ok());
        assert!(confirm_auth_method(&cli, None).is_err());
        assert!(confirm_auth_method(&cli, Some("OTHER")).is_err());
    }

    #[test]
    fn reauth_request_uses_exact_wire_lengths_and_method() {
        let args = [
            "sub",
            "--tls-password",
            "--topic",
            "p160/value",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--protocol-version",
            "5",
            "--auth-method",
            "plain",
            "--reauth-before-io",
        ];
        let cli = crate::cli::parse(args.into_iter().map(str::to_owned)).unwrap();
        let request = plain_reauth_request(&cli, "test-secret").unwrap();
        let mut wire = BytesMut::new();
        request.write(&mut wire).unwrap();
        assert_eq!(request.size(), wire.len());
        assert_eq!(
            wire.as_ref(),
            b"\xf0\x20\x19\x1e\x15\x00\x05PLAIN\x16\x00\x13\0device\0test-secret"
        );
        assert_eq!(
            Packet::read(&mut wire, Some(65536)).unwrap(),
            Packet::Auth(request)
        );
    }

    #[test]
    fn reauth_large_secret_uses_variable_length_packet_and_properties() {
        let args = [
            "sub",
            "--tls-password",
            "--topic",
            "p160/value",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--protocol-version",
            "5",
            "--auth-method",
            "plain",
            "--reauth-before-io",
        ];
        let cli = crate::cli::parse(args.into_iter().map(str::to_owned)).unwrap();
        let request = plain_reauth_request(&cli, &"s".repeat(200)).unwrap();
        let mut wire = BytesMut::new();
        request.write(&mut wire).unwrap();
        assert_ne!(wire[1] & 0x80, 0);
        assert_ne!(wire[4] & 0x80, 0);
        assert_eq!(request.size(), wire.len());
        assert_eq!(
            Packet::read(&mut wire, Some(65536)).unwrap(),
            Packet::Auth(request)
        );
    }
}
