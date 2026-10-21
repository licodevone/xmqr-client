//! MQTT 5 pub/sub with explicit, single-operation durable outbox support.
use std::{collections::HashMap, error::Error, io, time::Duration};

use rumqttc::v5::mqttbytes::v5::{
    Auth, AuthProperties, AuthReasonCode, ConnAck, ConnectReturnCode, DisconnectProperties, Filter,
    LastWill, LastWillProperties, PubAckReason, PubCompReason, PubRecReason, Publish,
    PublishProperties, RetainForwardRule, SubscribeProperties, SubscribeReasonCode, UnsubAckReason,
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

#[cfg(target_os = "linux")]
use crate::outbox::{
    InboundPublication, InboundStage, MAX_INBOUND_RECORDS, Operation, Outbox, PendingQos1,
    PendingQos2, PendingSubscription, Qos2Stage, Stage, SubscriptionSpec, SubscriptionStage,
};

#[cfg(target_os = "linux")]
enum DurableOutboxState {
    Qos1(Outbox, PendingQos1, bool),
    Qos2(Outbox, PendingQos2, bool),
}

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[cfg(test)]
mod audited_connack_tests {
    use super::{check_local_session, check_publish_capabilities};
    use bytes::BytesMut;
    use rumqttc::v5::mqttbytes::v5::{ConnAck, Packet};

    fn ack(max_qos: Option<u8>, retain_available: Option<u8>) -> ConnAck {
        let mut properties = Vec::new();
        if let Some(qos) = max_qos {
            properties.extend([0x24, qos]);
        }
        if let Some(available) = retain_available {
            properties.extend([0x25, available]);
        }
        let size = u8::try_from(properties.len()).unwrap();
        let mut wire = BytesMut::from(&[0x20, 3 + size, 0, 0, size][..]);
        wire.extend_from_slice(&properties);
        match Packet::read(&mut wire, Some(65536)).unwrap() {
            Packet::ConnAck(ack) => ack,
            _ => panic!("expected CONNACK"),
        }
    }

    #[test]
    fn publish_capabilities_enforce_boundaries_without_downgrading() {
        for qos in ["0", "1", "2"] {
            for retain in ["false", "true"] {
                let cli = crate::cli::parse(
                    [
                        "pub",
                        "--open-lab",
                        "--protocol-version",
                        "5",
                        "--topic",
                        "audit/t",
                        "--message",
                        "synthetic",
                        "--qos",
                        qos,
                        "--retain",
                        retain,
                    ]
                    .into_iter()
                    .map(str::to_owned),
                )
                .unwrap();
                for maximum in [None, Some(0), Some(1)] {
                    for available in [None, Some(0), Some(1)] {
                        let expected = qos.parse::<u8>().unwrap() <= maximum.unwrap_or(2)
                            && (retain == "false" || available != Some(0));
                        assert_eq!(
                            check_publish_capabilities(&cli, &ack(maximum, available), true)
                                .is_ok(),
                            expected
                        );
                        assert!(
                            check_publish_capabilities(&cli, &ack(maximum, available), false)
                                .is_ok()
                        );
                    }
                }
                assert!(check_publish_capabilities(&cli, &ack(Some(2), None), false).is_err());
                assert!(check_publish_capabilities(&cli, &ack(None, Some(2)), false).is_err());
            }
        }
    }

    #[test]
    fn session_present_requires_actual_local_state() {
        for present in [false, true] {
            let mut ack = ack(None, None);
            ack.session_present = present;
            for local in [false, true] {
                assert_eq!(check_local_session(&ack, local).is_ok(), !present || local);
            }
        }
    }
}

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
    let broker_address = cli.ws_url.as_deref().unwrap_or(&cli.host);
    let mut options = MqttOptions::new(&cli.client_id, broker_address, cli.port);
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    if crate::service_output::active() {
        options.set_manual_acks(true);
    }
    #[cfg(target_os = "linux")]
    if cli.subscription_state_dir.is_some() {
        options.set_manual_acks(true);
    }
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
    if let Some(url) = cli.ws_url.as_deref() {
        if url.starts_with("wss://") {
            options.set_transport(Transport::wss_with_config(
                tls::build_client_config(cli)?.into(),
            ));
        } else {
            options.set_transport(Transport::ws());
        }
    } else if matches!(
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

fn check_publish_capabilities(cli: &Cli, ack: &ConnAck, sends_publish: bool) -> AppResult<()> {
    let maximum_qos = ack.properties.as_ref().and_then(|p| p.max_qos);
    let retain_available = ack.properties.as_ref().and_then(|p| p.retain_available);
    if maximum_qos.is_some_and(|qos| qos > 1)
        || retain_available.is_some_and(|available| available > 1)
    {
        return Err(protocol_error("CONNACK MQTT 5 trouxe capacidade invalida").into());
    }
    // A journaled PUBREL completes an existing exchange; it sends no PUBLISH.
    if sends_publish {
        if cli.qos as u8 > maximum_qos.unwrap_or(2) {
            return Err(protocol_error("broker MQTT 5 nao suporta o QoS solicitado").into());
        }
        if cli.retain && retain_available == Some(0) {
            return Err(protocol_error("broker MQTT 5 nao suporta publicacao retained").into());
        }
    }
    Ok(())
}

fn check_local_session(ack: &ConnAck, has_local_session: bool) -> AppResult<()> {
    if ack.session_present && !has_local_session {
        return Err(protocol_error(
            "CONNACK MQTT 5 informou sessao anterior sem estado local; conexao encerrada",
        )
        .into());
    }
    Ok(())
}

fn check_publish_alias(cli: &Cli, broker_topic_alias_maximum: u16) -> AppResult<()> {
    if let Some(alias) = cli.publish_topic_alias
        && alias > broker_topic_alias_maximum
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("broker anuncia Topic Alias Maximum {broker_topic_alias_maximum}; alias solicitado {alias}"),
        )
        .into());
    }
    Ok(())
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

fn new_connection(cli: &Cli, options: MqttOptions) -> (AsyncClient, rumqttc::v5::EventLoop) {
    let durable_incoming = cfg!(target_os = "linux") && cli.subscription_state_dir.is_some();
    #[cfg(target_os = "linux")]
    let request_capacity = if durable_incoming {
        MAX_INBOUND_RECORDS
    } else {
        8
    };
    #[cfg(not(target_os = "linux"))]
    let request_capacity = 8;
    let (client, mut events) = AsyncClient::new(options, request_capacity);
    events.state.set_manual_incoming_qos2(durable_incoming);
    (client, events)
}

#[cfg(target_os = "linux")]
fn durable_session_flags(
    outbox: Option<&DurableOutboxState>,
    subscription: Option<&(Outbox, PendingSubscription, bool)>,
) -> (bool, bool) {
    let has_local_session = matches!(
        outbox,
        Some(DurableOutboxState::Qos1(_, _, true) | DurableOutboxState::Qos2(_, _, true))
    ) || subscription.is_some_and(|(_, _, existed)| *existed);
    let sends_publish = !matches!(outbox, Some(DurableOutboxState::Qos2(_, pending, _))
        if pending.stage == Qos2Stage::PubRelReady);
    (has_local_session, sends_publish)
}

#[cfg(target_os = "linux")]
async fn check_durable_session(
    ack: &ConnAck,
    has_local_session: bool,
    subscription: &mut Option<(Outbox, PendingSubscription, bool)>,
) -> AppResult<()> {
    if ack.session_present
        && !has_local_session
        && let Some((outbox, _, false)) = subscription.take()
    {
        // Preserve C108: discard only this newly prepared, never activated
        // declaration. Existing operator state is never removed here.
        disk_operation(move || outbox.remove_unactivated_subscription()).await?;
    }
    check_local_session(ack, has_local_session)
}

async fn run_inner(
    cli: &Cli,
    password: Option<String>,
    publication: Option<Publication>,
) -> AppResult<()> {
    let options = configure_options(cli, password.as_deref())?;
    #[cfg(target_os = "linux")]
    let outbox_state = if cli.outbox_dir.is_some() {
        Some(
            prepare_outbox(
                cli,
                publication
                    .as_ref()
                    .ok_or_else(|| protocol_error("payload ausente"))?,
            )
            .await?,
        )
    } else {
        None
    };
    #[cfg(target_os = "linux")]
    let mut subscription_state = if cli.subscription_state_dir.is_some() {
        Some(prepare_subscription_state(cli).await?)
    } else {
        None
    };
    #[cfg(target_os = "linux")]
    let (has_local_session, sends_publish) =
        durable_session_flags(outbox_state.as_ref(), subscription_state.as_ref());
    #[cfg(not(target_os = "linux"))]
    let (has_local_session, sends_publish) = (false, true);
    let sends_publish = sends_publish && matches!(&cli.mode, Mode::Pub { .. });
    let (client, mut events) = new_connection(cli, options);
    let limit = Duration::from_secs(u64::from(cli.timeout_seconds));
    let (broker_topic_alias_maximum, session_present) = loop {
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
                check_publish_capabilities(cli, &ack, sends_publish)?;
                #[cfg(target_os = "linux")]
                check_durable_session(&ack, has_local_session, &mut subscription_state).await?;
                #[cfg(not(target_os = "linux"))]
                check_local_session(&ack, has_local_session)?;
                break (
                    ack.properties
                        .as_ref()
                        .and_then(|properties| properties.topic_alias_max)
                        .unwrap_or(0),
                    ack.session_present,
                );
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
        return subscribe(
            cli,
            &client,
            &mut events,
            limit,
            #[cfg(target_os = "linux")]
            subscription_state,
            #[cfg(target_os = "linux")]
            session_present,
        )
        .await;
    };
    check_publish_alias(cli, broker_topic_alias_maximum)?;
    let publication = publication.ok_or_else(|| protocol_error("payload ausente"))?;
    #[cfg(target_os = "linux")]
    if let Some(state) = outbox_state {
        return publish_durable(cli, &client, &mut events, state, session_present, limit).await;
    }
    #[cfg(not(target_os = "linux"))]
    let _ = session_present;
    publish_regular(cli, &client, &mut events, &publication, topic, limit).await
}

#[cfg(target_os = "linux")]
async fn publish_durable(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    state: DurableOutboxState,
    session_present: bool,
    limit: Duration,
) -> AppResult<()> {
    match state {
        DurableOutboxState::Qos1(outbox, pending, existed) => {
            publish_from_outbox(
                cli,
                client,
                events,
                outbox,
                pending,
                existed,
                session_present,
                limit,
            )
            .await
        }
        DurableOutboxState::Qos2(outbox, pending, existed) => {
            publish_qos2_from_outbox(
                cli,
                client,
                events,
                outbox,
                pending,
                existed,
                session_present,
                limit,
            )
            .await
        }
    }
}

async fn publish_regular(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    publication: &Publication,
    topic: &str,
    limit: Duration,
) -> AppResult<()> {
    for index in 0..cli.repeat {
        if index > 0 {
            wait_repeat_interval(events, cli.repeat_interval_ms.unwrap()).await?;
        }
        let wire_topic = if index > 0 && cli.publish_topic_alias.is_some() {
            ""
        } else {
            topic
        };
        send_publication(
            client,
            cli,
            wire_topic,
            publication.first.clone(),
            cli.publish_topic_alias,
        )
        .await?;
        await_publish_complete(events, cli.qos, limit).await?;
        cli.output.complete(cli.qos);
    }
    request_disconnect(cli, client)?;
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

#[cfg(target_os = "linux")]
async fn disk_operation<T: Send + 'static>(
    action: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    tokio::task::spawn_blocking(action)
        .await
        .map_err(|_| io::Error::other("trabalhador da outbox falhou"))?
}

#[cfg(target_os = "linux")]
async fn prepare_outbox(cli: &Cli, publication: &Publication) -> io::Result<DurableOutboxState> {
    let Mode::Pub { topic, .. } = &cli.mode else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "outbox exige pub",
        ));
    };
    let (host, port) = durable_endpoint(cli);
    let operation = Operation {
        host,
        port,
        client_id: cli.client_id.clone(),
        topic: topic.clone(),
        payload: publication.first.clone(),
        retain: cli.retain,
        message_expiry: cli.message_expiry,
        payload_format: cli.payload_format,
        content_type: cli.content_type.clone(),
        response_topic: cli.response_topic.clone(),
        correlation_data: cli.correlation_data.clone(),
        user_properties: cli.publish_user_properties.clone(),
    };
    let directory = cli
        .outbox_dir
        .as_ref()
        .expect("CLI validated outbox")
        .clone();
    let qos = cli.qos;
    disk_operation(move || {
        let outbox = Outbox::open(&directory)?;
        if qos == QoS::ExactlyOnce {
            match outbox.load_qos2()? {
                Some(pending) if pending.operation == operation => {
                    Ok(DurableOutboxState::Qos2(outbox, pending, true))
                }
                Some(_) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "outbox pertence a outra publicacao; conteudo preservado",
                )),
                None => {
                    let pending = outbox.prepare_qos2(operation)?;
                    Ok(DurableOutboxState::Qos2(outbox, pending, false))
                }
            }
        } else {
            match outbox.load()? {
                Some(pending) if pending.operation == operation => {
                    Ok(DurableOutboxState::Qos1(outbox, pending, true))
                }
                Some(_) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "outbox pertence a outra publicacao; conteudo preservado",
                )),
                None => {
                    let pending = outbox.prepare(operation)?;
                    Ok(DurableOutboxState::Qos1(outbox, pending, false))
                }
            }
        }
    })
    .await
}

#[cfg(target_os = "linux")]
fn durable_endpoint(cli: &Cli) -> (String, u16) {
    cli.ws_url.as_deref().map_or_else(
        || (cli.host.clone(), cli.port),
        |url| {
            (
                url::Url::parse(url)
                    .expect("CLI validated WSS URL")
                    .to_string(),
                0,
            )
        },
    )
}

#[cfg(target_os = "linux")]
async fn prepare_subscription_state(cli: &Cli) -> io::Result<(Outbox, PendingSubscription, bool)> {
    let Mode::Sub { filters, .. } = &cli.mode else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "manifesto exige sub",
        ));
    };
    let (host, port) = durable_endpoint(cli);
    let spec = SubscriptionSpec {
        host,
        port,
        client_id: cli.client_id.clone(),
        username: cli.username.clone(),
        session_expiry: cli.session_expiry,
        filters: filters.clone(),
        qos: cli.qos as u8,
        retain_handling: cli.subscription_options.retain_handling,
        no_local: cli.subscription_options.no_local,
        retain_as_published: cli.subscription_options.retain_as_published,
        identifier: cli.subscription_options.identifier,
        user_properties: cli.subscribe_user_properties.clone(),
    };
    let directory = cli
        .subscription_state_dir
        .as_ref()
        .expect("CLI validated subscription state")
        .clone();
    disk_operation(move || {
        let outbox = Outbox::open(&directory)?;
        match outbox.load_subscription()? {
            Some(pending) if pending.spec == spec => {
                outbox.load_inbound()?; // corruption/identity errors precede network and ACKs
                Ok((outbox, pending, true))
            }
            Some(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "manifesto pertence a outra assinatura; conteudo preservado",
            )),
            None => {
                let pending = outbox.prepare_subscription(spec)?;
                Ok((outbox, pending, false))
            }
        }
    })
    .await
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
async fn publish_from_outbox(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    outbox: Outbox,
    pending: PendingQos1,
    existed: bool,
    session_present: bool,
    limit: Duration,
) -> AppResult<()> {
    if (existed && pending.stage == Stage::MayHaveSent && !session_present)
        || (!existed && session_present)
    {
        return Err(protocol_error(
            "broker nao confirmou sessao compativel; outbox preservada sem reenvio",
        )
        .into());
    }
    let retransmit = pending.stage == Stage::MayHaveSent;
    let (outbox, pending) = disk_operation(move || {
        let pending = outbox.mark_sending()?;
        Ok((outbox, pending))
    })
    .await?;
    let publish = durable_publish(
        pending.operation,
        rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
        pending.packet_id,
        retransmit,
    );
    client.publish_packet(publish).await?;
    let deadline = Instant::now() + limit;
    let reason = loop {
        let event = timeout_at(deadline, events.poll())
            .await
            .map_err(|_| protocol_error("PUBACK pendente; outbox preservada"))?
            .map_err(|_| protocol_error("conexao interrompida; outbox preservada"))?;
        match event {
            Event::Incoming(Incoming::PubAck(ack)) if ack.pkid == pending.packet_id => {
                break ack.reason;
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta inesperada; outbox preservada").into());
            }
        }
    };
    disk_operation(move || outbox.complete()).await?;
    if reason != PubAckReason::Success {
        return Err(puback_refusal(reason).into());
    }
    finish_durable_publish(cli, client, events, QoS::AtLeastOnce, limit).await
}

#[cfg(target_os = "linux")]
fn durable_publish(
    operation: Operation,
    qos: rumqttc::v5::mqttbytes::QoS,
    packet_id: u16,
    retransmit: bool,
) -> Publish {
    let has_properties = operation.message_expiry.is_some()
        || operation.payload_format.is_some()
        || operation.content_type.is_some()
        || operation.response_topic.is_some()
        || operation.correlation_data.is_some()
        || !operation.user_properties.is_empty();
    let properties = has_properties.then(|| PublishProperties {
        message_expiry_interval: operation.message_expiry,
        payload_format_indicator: operation.payload_format,
        content_type: operation.content_type,
        response_topic: operation.response_topic,
        correlation_data: operation.correlation_data.map(Into::into),
        user_properties: operation.user_properties,
        ..PublishProperties::default()
    });
    let mut publish = Publish::new(operation.topic, qos, operation.payload, properties);
    publish.pkid = packet_id;
    publish.dup = retransmit;
    publish.retain = operation.retain;
    publish
}

#[cfg(target_os = "linux")]
async fn finish_durable_publish(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    qos: QoS,
    limit: Duration,
) -> AppResult<()> {
    cli.output.complete(qos);
    request_disconnect(cli, client)?;
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado enviando DISCONNECT MQTT 5"))??;
        if matches!(event, Event::Outgoing(Outgoing::Disconnect)) {
            return Ok(());
        }
    }
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
async fn publish_qos2_from_outbox(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    outbox: Outbox,
    pending: PendingQos2,
    existed: bool,
    session_present: bool,
    limit: Duration,
) -> AppResult<()> {
    if (existed && pending.stage != Qos2Stage::Prepared && !session_present)
        || (!existed && session_present)
    {
        return Err(protocol_error(
            "broker nao confirmou sessao compativel; outbox QoS 2 preservada sem reenvio",
        )
        .into());
    }
    let packet_id = pending.packet_id;
    let outbox = if pending.stage == Qos2Stage::PubRelReady {
        outbox
    } else {
        let retransmit = pending.stage == Qos2Stage::PublishMayHaveSent;
        let (outbox, pending) = disk_operation(move || {
            let pending = outbox.mark_qos2_sending()?;
            Ok((outbox, pending))
        })
        .await?;
        let publish = durable_publish(
            pending.operation,
            rumqttc::v5::mqttbytes::QoS::ExactlyOnce,
            packet_id,
            retransmit,
        );
        client.publish_packet_manual_qos2(publish).await?;
        let deadline = Instant::now() + limit;
        let reason = loop {
            let event = timeout_at(deadline, events.poll())
                .await
                .map_err(|_| protocol_error("PUBREC pendente; outbox QoS 2 preservada"))?
                .map_err(|_| protocol_error("conexao interrompida; outbox QoS 2 preservada"))?;
            match event {
                Event::Incoming(Incoming::PubRec(ack)) if ack.pkid == packet_id => {
                    break ack.reason;
                }
                Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
                Event::Incoming(_) => {
                    return Err(
                        protocol_error("resposta inesperada; outbox QoS 2 preservada").into(),
                    );
                }
            }
        };
        if !matches!(
            reason,
            PubRecReason::Success | PubRecReason::NoMatchingSubscribers
        ) {
            disk_operation(move || outbox.complete_qos2()).await?;
            return Err(pubrec_refusal(reason).into());
        }
        let (outbox, _) = disk_operation(move || {
            let pending = outbox.mark_pubrel_ready()?;
            Ok((outbox, pending))
        })
        .await?;
        outbox
    };
    client.pubrel_packet(packet_id).await?;
    let deadline = Instant::now() + limit;
    let reason = loop {
        let event = timeout_at(deadline, events.poll())
            .await
            .map_err(|_| protocol_error("PUBCOMP pendente; outbox QoS 2 preservada"))?
            .map_err(|_| protocol_error("conexao interrompida; outbox QoS 2 preservada"))?;
        match event {
            Event::Incoming(Incoming::PubComp(ack)) if ack.pkid == packet_id => {
                break ack.reason;
            }
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_)) => {}
            Event::Incoming(_) => {
                return Err(protocol_error("resposta inesperada; outbox QoS 2 preservada").into());
            }
        }
    };
    disk_operation(move || outbox.complete_qos2()).await?;
    if reason != PubCompReason::Success {
        return Err(pubcomp_refusal(reason).into());
    }
    finish_durable_publish(cli, client, events, QoS::ExactlyOnce, limit).await
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

fn display_incoming(cli: &Cli, publish: &Publish, topic: &str) -> AppResult<bool> {
    let Mode::Sub { exclusions, .. } = &cli.mode else {
        return Err(protocol_error("assinatura esperada").into());
    };
    if !cli.retained_filter.accepts(publish.retain)
        || exclusions
            .iter()
            .any(|filter| crate::cli::matches_filter(filter, topic))
    {
        return Ok(false);
    }
    let qos = match publish.qos {
        rumqttc::v5::mqttbytes::QoS::AtMostOnce => QoS::AtMostOnce,
        rumqttc::v5::mqttbytes::QoS::AtLeastOnce => QoS::AtLeastOnce,
        rumqttc::v5::mqttbytes::QoS::ExactlyOnce => QoS::ExactlyOnce,
    };
    let mut display = rumqttc::mqttbytes::v4::Publish::new(topic, qos, publish.payload.clone());
    display.retain = publish.retain;
    cli.output
        .publish_mqtt5(&display, publish.properties.as_ref())?;
    Ok(true)
}

#[cfg(target_os = "linux")]
async fn journal_incoming(
    cli: &Cli,
    client: &AsyncClient,
    outbox: Outbox,
    publish: &Publish,
    topic: &str,
) -> AppResult<(Outbox, bool)> {
    let message = InboundPublication::from_publish(publish, topic)?;
    let (outbox, pending) = disk_operation(move || {
        let pending = outbox.receive_inbound(message)?;
        Ok((outbox, pending))
    })
    .await?;
    let mut displayed = false;
    let outbox = if pending.stage == InboundStage::Received {
        // If the process dies between flush and the next durable transition,
        // recovery can duplicate stdout. No exactly-once consumer claim is made.
        displayed = display_incoming(cli, &pending.publication.publication()?, topic)?;
        disk_operation(move || {
            outbox.inbound_output_ready(pending.publication.packet_id)?;
            Ok(outbox)
        })
        .await?
    } else {
        outbox
    };
    // The caller also owns EventLoop::poll: waiting on its full request channel
    // would deadlock before the peer's ACK deadline. State remains durable.
    client.try_ack(publish)?;
    Ok((outbox, displayed))
}

#[cfg(target_os = "linux")]
async fn journal_pubrel(client: &AsyncClient, outbox: Outbox, packet_id: u16) -> AppResult<Outbox> {
    let (outbox, known) = disk_operation(move || {
        let known = outbox.inbound_pubrel(packet_id)?;
        Ok((outbox, known))
    })
    .await?;
    let reason = if known {
        PubCompReason::Success
    } else {
        PubCompReason::PacketIdentifierNotFound
    };
    client.try_pubcomp_packet(packet_id, reason)?;
    Ok(outbox)
}

#[cfg(target_os = "linux")]
async fn journal_ack_written(outbox: Outbox, packet_id: u16, qos: u8) -> io::Result<Outbox> {
    disk_operation(move || {
        outbox.inbound_ack_written(packet_id, qos)?;
        Ok(outbox)
    })
    .await
}

#[cfg(target_os = "linux")]
async fn start_durable_subscription(
    cli: &Cli,
    client: &AsyncClient,
    filters: &[String],
    qos: rumqttc::v5::mqttbytes::QoS,
    state: Option<(Outbox, PendingSubscription, bool)>,
    session_present: bool,
) -> AppResult<(bool, Option<Outbox>)> {
    let Some((outbox, pending, existed)) = state else {
        send_subscribe(
            client,
            filters,
            qos,
            cli.subscription_options,
            &cli.subscribe_user_properties,
        )
        .await?;
        return Ok((false, None));
    };
    if existed && pending.stage == SubscriptionStage::Active {
        if !session_present {
            return Err(protocol_error(
                "broker nao confirmou sessao de assinatura; manifesto preservado sem re-assinar",
            )
            .into());
        }
        cli.output
            .status("Assinatura retomada (MQTT 5). Aguardando mensagens; Ctrl+C para sair.");
        return Ok((true, Some(outbox)));
    }
    if !existed && session_present {
        disk_operation(move || outbox.remove_unactivated_subscription()).await?;
        return Err(protocol_error(
            "sessao existente antes da primeira assinatura; nenhum SUBSCRIBE enviado",
        )
        .into());
    }
    send_subscribe(
        client,
        filters,
        qos,
        cli.subscription_options,
        &cli.subscribe_user_properties,
    )
    .await?;
    Ok((false, Some(outbox)))
}

#[allow(clippy::too_many_lines)] // SUBACK commit and message output share one event loop.
async fn subscribe(
    cli: &Cli,
    client: &AsyncClient,
    events: &mut rumqttc::v5::EventLoop,
    limit: Duration,
    #[cfg(target_os = "linux")] subscription_state: Option<(Outbox, PendingSubscription, bool)>,
    #[cfg(target_os = "linux")] session_present: bool,
) -> AppResult<()> {
    let Mode::Sub {
        filters,
        exclusions: _,
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
    #[cfg(target_os = "linux")]
    let (mut acknowledged, mut subscription_journal) = start_durable_subscription(
        cli,
        client,
        filters,
        qos,
        subscription_state,
        session_present,
    )
    .await?;
    #[cfg(not(target_os = "linux"))]
    let mut acknowledged = {
        send_subscribe(
            client,
            filters,
            qos,
            cli.subscription_options,
            &cli.subscribe_user_properties,
        )
        .await?;
        false
    };
    let mut received = 0_u64;
    #[cfg(all(windows, feature = "windows-service-runtime"))]
    let mut service_qos2 = std::collections::HashSet::new();
    let mut aliases = HashMap::new();
    #[cfg(target_os = "linux")]
    let mut ack_deadlines = if let Some(outbox) = subscription_journal.take() {
        let (outbox, pending) = disk_operation(move || {
            let pending = outbox.load_inbound()?;
            Ok((outbox, pending))
        })
        .await?;
        subscription_journal = Some(outbox);
        pending
            .into_iter()
            .filter(|entry| entry.stage == InboundStage::AwaitPubRel)
            .map(|entry| (entry.publication.packet_id, Instant::now() + limit))
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };
    loop {
        #[cfg(all(windows, feature = "windows-service-runtime"))]
        let service_incomplete = !service_qos2.is_empty();
        #[cfg(not(all(windows, feature = "windows-service-runtime")))]
        let service_incomplete = false;
        if acknowledged && !service_incomplete && count.is_some_and(|target| received >= target) {
            finish_counted_subscription(cli, client, events, filters, limit).await?;
            return Ok(());
        }
        let wait = if acknowledged && !service_incomplete {
            Duration::from_secs(90)
        } else {
            limit
        };
        let deadline = Instant::now() + wait;
        #[cfg(target_os = "linux")]
        let deadline = ack_deadlines
            .values()
            .min()
            .copied()
            .unwrap_or(deadline)
            .min(deadline);
        if Instant::now() >= deadline {
            return Err(protocol_error("tempo esgotado completando ACK inbound MQTT 5").into());
        }
        let event = timeout_at(deadline, events.poll())
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
                #[cfg(target_os = "linux")]
                if let Some(outbox) = subscription_journal.take() {
                    let outbox = disk_operation(move || {
                        outbox.activate_subscription()?;
                        Ok(outbox)
                    })
                    .await?;
                    subscription_journal = Some(outbox);
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
                #[cfg(target_os = "linux")]
                if publish.qos != rumqttc::v5::mqttbytes::QoS::AtMostOnce
                    && let Some(outbox) = subscription_journal.take()
                {
                    let (outbox, displayed) =
                        journal_incoming(cli, client, outbox, &publish, &topic).await?;
                    subscription_journal = Some(outbox);
                    ack_deadlines
                        .entry(publish.pkid)
                        .or_insert_with(|| Instant::now() + limit);
                    received += u64::from(displayed);
                    continue;
                }
                if count.is_none_or(|target| received < target)
                    && display_incoming(cli, &publish, &topic)?
                {
                    received += 1;
                }
                #[cfg(all(windows, feature = "windows-service-runtime"))]
                if crate::service_output::active() {
                    client.try_ack(&publish)?;
                    if publish.qos == rumqttc::v5::mqttbytes::QoS::ExactlyOnce {
                        service_qos2.insert(publish.pkid);
                    }
                }
            }
            #[cfg(all(windows, feature = "windows-service-runtime"))]
            Event::Incoming(Incoming::PubRel(rel)) if crate::service_output::active() => {
                service_qos2.remove(&rel.pkid);
            }
            #[cfg(target_os = "linux")]
            Event::Incoming(Incoming::PubRel(rel)) if subscription_journal.is_some() => {
                let outbox = subscription_journal.take().expect("guard checked journal");
                subscription_journal = Some(journal_pubrel(client, outbox, rel.pkid).await?);
                ack_deadlines
                    .entry(rel.pkid)
                    .or_insert_with(|| Instant::now() + limit);
            }
            #[cfg(target_os = "linux")]
            Event::Outgoing(Outgoing::PubAck(id)) if subscription_journal.is_some() => {
                let outbox = subscription_journal.take().expect("guard checked journal");
                subscription_journal = Some(journal_ack_written(outbox, id, 1).await?);
                ack_deadlines.remove(&id);
            }
            #[cfg(target_os = "linux")]
            Event::Outgoing(Outgoing::PubComp(id)) if subscription_journal.is_some() => {
                let outbox = subscription_journal.take().expect("guard checked journal");
                subscription_journal = Some(journal_ack_written(outbox, id, 2).await?);
                ack_deadlines.remove(&id);
            }
            // Default mode's state machine already validates PUBREL and sends
            // PUBCOMP. An unsolicited identifier still fails inside poll().
            Event::Outgoing(_) | Event::Incoming(Incoming::PingResp(_) | Incoming::PubRel(_)) => {}
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
    request_disconnect(cli, client)?;
    loop {
        let event = timeout(limit, events.poll())
            .await
            .map_err(|_| protocol_error("tempo esgotado enviando DISCONNECT MQTT 5"))??;
        if matches!(event, Event::Outgoing(Outgoing::Disconnect)) {
            return Ok(());
        }
    }
}

fn request_disconnect(cli: &Cli, client: &AsyncClient) -> AppResult<()> {
    if let Some(seconds) = cli.disconnect_session_expiry {
        client.try_disconnect_with_properties(DisconnectProperties {
            session_expiry_interval: Some(seconds),
            reason_string: None,
            user_properties: Vec::new(),
            server_reference: None,
        })?;
    } else {
        client.try_disconnect()?;
    }
    Ok(())
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

#[cfg(all(test, target_os = "linux"))]
mod subscription_ack_tests {
    use super::{configure_options, durable_endpoint};

    #[test]
    fn durable_wss_endpoint_identity_is_distinct_from_tcp_and_other_urls() {
        let base = [
            "pub",
            "--topic",
            "sensor/value",
            "--message",
            "ok",
            "--username",
            "device",
            "--ca",
            "ca.crt",
            "--cert",
            "client.crt",
            "--key",
            "client.key",
            "--password-file",
            "/tmp/password",
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--client-id",
            "fixed",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "60",
            "--outbox-dir",
            "/tmp/private",
        ];
        let parse = |extra: &[&str]| {
            crate::cli::parse(
                base.into_iter()
                    .chain(extra.iter().copied())
                    .map(str::to_owned),
            )
            .unwrap()
        };
        let tcp = parse(&["--host", "127.0.0.1", "--port", "8883"]);
        let local = parse(&["--ws-url", "wss://127.0.0.1:8443/mqtt"]);
        let other = parse(&["--ws-url", "wss://127.0.0.1:8444/mqtt"]);
        assert_eq!(durable_endpoint(&tcp), ("127.0.0.1".to_owned(), 8883));
        assert_eq!(
            durable_endpoint(&local),
            ("wss://127.0.0.1:8443/mqtt".to_owned(), 0)
        );
        assert_ne!(durable_endpoint(&tcp), durable_endpoint(&local));
        assert_ne!(durable_endpoint(&local), durable_endpoint(&other));
    }

    #[test]
    fn manual_acks_are_opt_in_for_persistent_subscription_only() {
        let base = [
            "sub",
            "--open-lab",
            "--topic",
            "sensor/#",
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--client-id",
            "persistent-sub",
            "--clean-start",
            "false",
            "--session-expiry-seconds",
            "60",
        ];
        let ordinary = crate::cli::parse(base.into_iter().map(str::to_owned)).unwrap();
        assert!(!configure_options(&ordinary, None).unwrap().manual_acks());
        let durable = crate::cli::parse(
            base.into_iter()
                .chain(["--subscription-state-dir", "/tmp/c110-fixture"])
                .map(str::to_owned),
        )
        .unwrap();
        assert!(configure_options(&durable, None).unwrap().manual_acks());
    }
}
