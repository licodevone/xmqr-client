//! Metadados de aplicação, prazo e percurso do modo MQTT 5 da bridge.
use rumqttc::v5::mqttbytes::v5::{Publish, PublishProperties};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{AppResult, Config, invalid, mapped_topic, valid_id, valid_topic};

pub const PATH_PROPERTY: &str = "xmqr.bridge.path";

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResponsePolicy {
    Preserve,
    MapPrefix,
}

#[derive(Clone)]
pub struct Settings {
    pub bridge_id: String,
    pub max_hops: usize,
    pub response_policy: ResponsePolicy,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub format: Option<u8>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation: Option<Vec<u8>>,
    pub users: Vec<(String, String)>,
    pub expires_ms: Option<u64>,
}

impl Metadata {
    pub fn validate(&self) -> AppResult<()> {
        let valid_text = |text: &str| text.len() <= 256 && !text.contains('\0');
        if self.format.is_some_and(|value| value > 1)
            || self
                .content_type
                .as_deref()
                .is_some_and(|value| !valid_text(value))
            || self
                .response_topic
                .as_deref()
                .is_some_and(|value| !valid_topic(value))
            || self
                .correlation
                .as_ref()
                .is_some_and(|value| value.len() > 256)
            || self.users.len() > 16
            || self
                .users
                .iter()
                .any(|(key, value)| !valid_text(key) || !valid_text(value))
        {
            return Err(invalid("metadados da bridge invalidos ou acima do limite").into());
        }
        Ok(())
    }

    pub fn properties(&self, now_ms: u64) -> AppResult<Option<PublishProperties>> {
        self.validate()?;
        let expiry = match self.expires_ms {
            Some(deadline) if deadline.saturating_sub(now_ms) < 1000 => return Ok(None),
            Some(deadline) => Some(u32::try_from((deadline - now_ms) / 1000)?),
            None => None,
        };
        Ok(Some(PublishProperties {
            payload_format_indicator: self.format,
            message_expiry_interval: expiry,
            response_topic: self.response_topic.clone(),
            correlation_data: self.correlation.clone().map(Into::into),
            user_properties: self.users.clone(),
            content_type: self.content_type.clone(),
            // Alias e Subscription Identifier pertencem à conexão de origem.
            ..Default::default()
        }))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub source_topic: String,
    pub target_topic: String,
    pub payload: Vec<u8>,
    pub source_qos: u8,
    pub source_pkid: u16,
    pub retain: bool,
    pub fingerprint: String,
    pub metadata: Metadata,
    pub discard: bool,
}

fn application(message: &Publish, now_ms: u64) -> AppResult<Metadata> {
    let properties = message.properties.clone().unwrap_or_default();
    let result = Metadata {
        format: properties.payload_format_indicator,
        content_type: properties.content_type,
        response_topic: properties.response_topic,
        correlation: properties.correlation_data.map(|bytes| bytes.to_vec()),
        users: properties.user_properties,
        expires_ms: properties
            .message_expiry_interval
            .map(|seconds| now_ms.saturating_add(u64::from(seconds) * 1000)),
    };
    result.validate()?;
    if result.format == Some(1) && std::str::from_utf8(&message.payload).is_err() {
        return Err(invalid("payload nao corresponde ao formato UTF-8 declarado").into());
    }
    Ok(result)
}

pub fn fingerprint(message: &Publish) -> AppResult<String> {
    let mut metadata = application(message, 0)?;
    // O broker desconta expiry em redelivery; o restante identifica a mensagem.
    metadata.expires_ms = None;
    let bytes = serde_json::to_vec(&(
        message.topic.as_ref(),
        message.payload.as_ref(),
        message.qos as u8,
        message.retain,
        metadata,
    ))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn envelope(
    config: &Config,
    settings: &Settings,
    message: &Publish,
    now_ms: u64,
) -> AppResult<Envelope> {
    let source_topic = std::str::from_utf8(&message.topic)?;
    if message.qos as u8 > config.route.qos
        || message.payload.len() > super::MAX_PAYLOAD_BYTES
        || (message.qos as u8 > 0 && message.pkid == 0)
    {
        return Err(invalid("publicacao da origem invalida").into());
    }
    let target_topic = mapped_topic(&config.route, source_topic)
        .ok_or_else(|| invalid("topico da origem fora da rota"))?;
    let mut metadata = application(message, now_ms)?;
    let fingerprint = fingerprint(message)?;
    if let (ResponsePolicy::MapPrefix, Some(topic)) =
        (settings.response_policy, &metadata.response_topic)
    {
        metadata.response_topic = Some(
            mapped_topic(&config.route, topic)
                .ok_or_else(|| invalid("response topic fora do prefixo configurado"))?,
        );
    }
    let markers = metadata
        .users
        .iter()
        .filter(|(key, _)| key == PATH_PROPERTY)
        .collect::<Vec<_>>();
    if markers.len() > 1 {
        return Err(invalid("marcador de percurso repetido").into());
    }
    let mut path = if let Some((_, value)) = markers.first() {
        value.split(',').map(str::to_owned).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if path.len() > 8
        || path
            .iter()
            .enumerate()
            .any(|(index, id)| !valid_id(id) || id.len() > 24 || path[..index].contains(id))
    {
        return Err(invalid("marcador de percurso invalido").into());
    }
    let discard = path.contains(&settings.bridge_id) || path.len() >= settings.max_hops;
    metadata.users.retain(|(key, _)| key != PATH_PROPERTY);
    if !discard {
        path.push(settings.bridge_id.clone());
        metadata.users.push((PATH_PROPERTY.into(), path.join(",")));
    }
    metadata.validate()?;
    Ok(Envelope {
        source_topic: source_topic.into(),
        target_topic,
        payload: message.payload.to_vec(),
        source_qos: message.qos as u8,
        source_pkid: message.pkid,
        retain: message.retain,
        fingerprint,
        metadata,
        discard,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{Endpoint, Route, SourceProtocol};
    use super::*;
    use rumqttc::v5::mqttbytes::QoS;
    use std::path::PathBuf;

    fn config() -> Config {
        let endpoint = |id: &str, port| Endpoint {
            broker_id: None,
            host: "127.0.0.1".into(),
            port,
            client_id: id.into(),
            username: "fixture".into(),
            password_file: PathBuf::from("/fixture/password"),
            ca: PathBuf::from("/fixture/ca.crt"),
            cert: PathBuf::from("/fixture/client.crt"),
            key: PathBuf::from("/fixture/client.key"),
        };
        Config {
            version: 1,
            source_protocol: SourceProtocol::Mqtt5,
            source: endpoint("origin", 1883),
            destination: endpoint("target", 1884),
            route: Route {
                source_prefix: "src/".into(),
                target_prefix: "dst/".into(),
                qos: 2,
            },
            topology_file: None,
            link_id: None,
            journal_dir: None,
            max_retries: 1,
        }
    }

    fn settings() -> Settings {
        Settings {
            bridge_id: "unit-bridge".into(),
            max_hops: 4,
            response_policy: ResponsePolicy::MapPrefix,
        }
    }

    #[test]
    fn publish_real_preserva_aplicacao_remove_props_da_conexao_e_estabiliza_fingerprint() {
        let properties = PublishProperties {
            payload_format_indicator: Some(1),
            message_expiry_interval: Some(30),
            content_type: Some("application/json".into()),
            response_topic: Some("src/replies".into()),
            correlation_data: Some(vec![0, 255, 128, 7].into()),
            user_properties: vec![
                ("trace".into(), "first".into()),
                (PATH_PROPERTY.into(), "external".into()),
                ("trace".into(), "second".into()),
            ],
            topic_alias: Some(7),
            subscription_identifiers: vec![12, 19],
        };
        let payload = b"{\"temperature\":24}".to_vec();
        let mut message = Publish::new(
            "src/readings",
            QoS::AtLeastOnce,
            payload.clone(),
            Some(properties),
        );
        message.pkid = 17;
        message.retain = true;
        let transformed = envelope(&config(), &settings(), &message, 1000).unwrap();
        assert_eq!(transformed.source_topic, "src/readings");
        assert_eq!(transformed.target_topic, "dst/readings");
        assert_eq!(transformed.source_qos, 1);
        assert_eq!(transformed.source_pkid, 17);
        assert_eq!(transformed.payload, payload);
        assert!(transformed.retain);
        assert!(!transformed.discard);
        assert_eq!(transformed.metadata.expires_ms, Some(31_000));
        let outbound = transformed.metadata.properties(3500).unwrap().unwrap();
        assert_eq!(outbound.payload_format_indicator, Some(1));
        assert_eq!(outbound.content_type.as_deref(), Some("application/json"));
        assert_eq!(outbound.response_topic.as_deref(), Some("dst/replies"));
        assert_eq!(outbound.message_expiry_interval, Some(27));
        assert_eq!(
            outbound.correlation_data.unwrap().as_ref(),
            &[0, 255, 128, 7]
        );
        assert_eq!(
            outbound.user_properties,
            vec![
                ("trace".into(), "first".into()),
                ("trace".into(), "second".into()),
                (PATH_PROPERTY.into(), "external,unit-bridge".into()),
            ]
        );
        assert!(outbound.topic_alias.is_none());
        assert_eq!(outbound.subscription_identifiers, [] as [usize; 0]);

        let mut redelivery = message.clone();
        {
            let properties = redelivery.properties.as_mut().unwrap();
            properties.topic_alias = Some(99);
            properties.subscription_identifiers = vec![3];
            properties.message_expiry_interval = Some(28);
        }
        assert_eq!(
            fingerprint(&message).unwrap(),
            fingerprint(&redelivery).unwrap()
        );
        redelivery.properties.as_mut().unwrap().user_properties[0].1 =
            "changed-application-property".into();
        assert_ne!(
            fingerprint(&message).unwrap(),
            fingerprint(&redelivery).unwrap()
        );
    }

    #[test]
    fn response_topic_segue_1024_e_rejeita_estouro_apos_mapeamento() {
        let mut message = Publish::new(
            "src/readings",
            QoS::AtMostOnce,
            Vec::new(),
            Some(PublishProperties {
                response_topic: Some("src/".to_owned() + &"r".repeat(1020)),
                ..Default::default()
            }),
        );
        let transformed = envelope(&config(), &settings(), &message, 1000).unwrap();
        assert_eq!(transformed.metadata.response_topic.unwrap().len(), 1024);
        message.properties.as_mut().unwrap().response_topic =
            Some("src/".to_owned() + &"r".repeat(1021));
        assert!(envelope(&config(), &settings(), &message, 1000).is_err());

        let mut expanded = config();
        expanded.route.target_prefix = "destination/".into();
        let maximum_source =
            1024 - expanded.route.target_prefix.len() + expanded.route.source_prefix.len();
        message.properties.as_mut().unwrap().response_topic =
            Some("src/".to_owned() + &"r".repeat(maximum_source - 4));
        let transformed = envelope(&expanded, &settings(), &message, 1000).unwrap();
        assert_eq!(transformed.metadata.response_topic.unwrap().len(), 1024);
        message
            .properties
            .as_mut()
            .unwrap()
            .response_topic
            .as_mut()
            .unwrap()
            .push('x');
        assert!(envelope(&expanded, &settings(), &message, 1000).is_err());
    }

    #[test]
    fn campos_aplicacao_limitam_256_sem_reduzir_response_topic() {
        let metadata = Metadata {
            content_type: Some("t".repeat(256)),
            correlation: Some(vec![0; 256]),
            users: vec![("k".repeat(256), "v".repeat(256))],
            response_topic: Some("r".repeat(1024)),
            ..Default::default()
        };
        assert!(metadata.validate().is_ok());
        let mut oversized = metadata.clone();
        oversized.content_type.as_mut().unwrap().push('x');
        assert!(oversized.validate().is_err());
        let mut oversized = metadata.clone();
        oversized.correlation.as_mut().unwrap().push(0);
        assert!(oversized.validate().is_err());
        let mut oversized = metadata.clone();
        oversized.users[0].0.push('x');
        assert!(oversized.validate().is_err());
        let mut oversized = metadata;
        oversized.users[0].1.push('x');
        assert!(oversized.validate().is_err());
    }

    #[test]
    fn expiry_nao_renova_e_propriedades_de_conexao_nao_sao_reenviadas() {
        let metadata = Metadata {
            expires_ms: Some(5500),
            users: vec![("a".into(), "1".into()), ("a".into(), "2".into())],
            correlation: Some(vec![0, 255]),
            ..Default::default()
        };
        let properties = metadata.properties(2000).unwrap().unwrap();
        assert_eq!(properties.message_expiry_interval, Some(3));
        assert_eq!(properties.user_properties, metadata.users);
        assert_eq!(properties.correlation_data.unwrap().as_ref(), &[0, 255]);
        assert!(properties.topic_alias.is_none());
        assert_eq!(properties.subscription_identifiers, [] as [usize; 0]);
        assert!(metadata.properties(4501).unwrap().is_none());
    }

    #[test]
    fn metadados_recusam_excesso_sem_truncar() {
        let mut metadata = Metadata {
            users: vec![("a".into(), "b".into()); 17],
            ..Default::default()
        };
        assert!(metadata.validate().is_err());
        metadata.users.clear();
        metadata.correlation = Some(vec![0; 257]);
        assert!(metadata.validate().is_err());
    }
}
