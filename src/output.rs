use std::{
    fmt::Write as _,
    io::{self, Write as _},
};

use rumqttc::v5::mqttbytes::v5::PublishProperties;
use rumqttc::{QoS, mqttbytes::v4::Publish};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Text,
    Jsonl,
    Hex,
    Raw,
}

impl Output {
    pub fn publish(self, publish: &Publish) -> io::Result<()> {
        let mut stdout = io::stdout().lock();
        if self == Self::Raw {
            stdout.write_all(&publish.payload)?;
        } else {
            stdout.write_all(self.render_publish(publish).as_bytes())?;
            stdout.write_all(b"\n")?;
        }
        stdout.flush()
    }

    pub fn publish_mqtt5(
        self,
        publish: &Publish,
        properties: Option<&PublishProperties>,
    ) -> io::Result<()> {
        if self != Self::Jsonl {
            return self.publish(publish);
        }
        let mut rendered = self.render_publish(publish);
        if let Some(metadata) = properties.and_then(render_mqtt5_metadata) {
            rendered.pop(); // Replace the closing JSON object brace.
            rendered.push_str(",\"metadata\":");
            rendered.push_str(&metadata);
            rendered.push('}');
        }
        let mut stdout = io::stdout().lock();
        stdout.write_all(rendered.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()
    }

    pub fn status(self, message: &str) {
        match self {
            Self::Text => println!("{message}"),
            Self::Jsonl | Self::Hex | Self::Raw => eprintln!("{message}"),
        }
    }

    pub fn complete(self, qos: QoS) {
        match self {
            Self::Jsonl => println!(
                "{{\"event\":\"publish_complete\",\"qos\":{},\"confirmation\":\"{}\"}}",
                qos_number(qos),
                if qos == QoS::AtMostOnce {
                    "sent"
                } else {
                    "broker_protocol_ack"
                }
            ),
            Self::Text if qos == QoS::AtMostOnce => {
                println!("PUBLISH QoS 0 enviado; o protocolo nao confirma entrega ao assinante.");
            }
            Self::Text => println!(
                "Publicacao confirmada pelo broker em QoS {}.",
                qos_number(qos)
            ),
            Self::Hex | Self::Raw => eprintln!("Publicacao confirmada pelo broker."),
        }
    }

    fn render_publish(self, publish: &Publish) -> String {
        if self == Self::Text {
            return format!(
                "topico={:?} qos={} retain={} mensagem={:?}",
                publish.topic,
                qos_number(publish.qos),
                publish.retain,
                String::from_utf8_lossy(&publish.payload)
            );
        }
        if self == Self::Hex {
            let mut topic = String::with_capacity(publish.topic.len() * 2);
            let mut payload = String::with_capacity(publish.payload.len() * 2);
            for byte in publish.topic.as_bytes() {
                write!(topic, "{byte:02x}").unwrap();
            }
            for byte in &publish.payload {
                write!(payload, "{byte:02x}").unwrap();
            }
            return format!(
                "topic_hex={topic} payload_hex={payload} qos={} retain={}",
                qos_number(publish.qos),
                publish.retain
            );
        }
        assert_ne!(self, Self::Raw, "raw output does not render text");
        let bytes = publish
            .payload
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"event\":\"publish\",\"topic\":{},\"qos\":{},\"retain\":{},\"payload_bytes\":[{}]}}",
            json_string(&publish.topic),
            qos_number(publish.qos),
            publish.retain,
            bytes
        )
    }
}

fn render_mqtt5_metadata(properties: &PublishProperties) -> Option<String> {
    let mut fields = Vec::new();
    if !properties.subscription_identifiers.is_empty() {
        let ids = properties
            .subscription_identifiers
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",");
        fields.push(format!("\"subscription_identifiers\":[{ids}]"));
    }
    if let Some(value) = properties.payload_format_indicator {
        fields.push(format!("\"payload_format_indicator\":{value}"));
    }
    if let Some(value) = properties.message_expiry_interval {
        fields.push(format!("\"message_expiry_interval\":{value}"));
    }
    if let Some(value) = &properties.content_type {
        fields.push(format!("\"content_type\":{}", json_string(value)));
    }
    if let Some(value) = &properties.response_topic {
        fields.push(format!("\"response_topic\":{}", json_string(value)));
    }
    if let Some(value) = &properties.correlation_data {
        let mut hex = String::with_capacity(value.len() * 2);
        for byte in value {
            write!(hex, "{byte:02x}").unwrap();
        }
        fields.push(format!("\"correlation_data_hex\":\"{hex}\""));
    }
    if !properties.user_properties.is_empty() {
        let pairs = properties
            .user_properties
            .iter()
            .map(|(key, value)| format!("[{},{}]", json_string(key), json_string(value)))
            .collect::<Vec<_>>()
            .join(",");
        fields.push(format!("\"user_properties\":[{pairs}]"));
    }
    (!fields.is_empty()).then(|| format!("{{{}}}", fields.join(",")))
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => write!(result, "\\u{:04x}", u32::from(ch)).unwrap(),
            _ => result.push(ch),
        }
    }
    result.push('"');
    result
}

pub const fn qos_number(qos: QoS) -> u8 {
    match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_json_is_lossless_and_escaped() {
        let mut publish = Publish::new("a\n\"\\é", QoS::ExactlyOnce, [0, 255, 10, 34]);
        publish.retain = true;
        assert_eq!(
            Output::Jsonl.render_publish(&publish),
            "{\"event\":\"publish\",\"topic\":\"a\\u000a\\\"\\\\é\",\"qos\":2,\"retain\":true,\"payload_bytes\":[0,255,10,34]}"
        );
        assert_eq!(
            json_string("\u{0}\t\r\u{1f}"),
            "\"\\u0000\\u0009\\u000d\\u001f\""
        );
    }

    #[test]
    fn empty_payload_and_text_are_compatible() {
        let publish = Publish::new("t", QoS::AtMostOnce, Vec::<u8>::new());
        assert!(
            Output::Jsonl
                .render_publish(&publish)
                .ends_with("\"payload_bytes\":[]}")
        );
        assert_eq!(
            Output::Text.render_publish(&publish),
            "topico=\"t\" qos=0 retain=false mensagem=\"\""
        );
    }

    #[test]
    fn hex_is_reversible_for_topic_and_binary_payload() {
        let mut publish = Publish::new("é/valor", QoS::AtLeastOnce, [0, 255, 10, 34]);
        publish.retain = true;
        assert_eq!(
            Output::Hex.render_publish(&publish),
            "topic_hex=c3a92f76616c6f72 payload_hex=00ff0a22 qos=1 retain=true"
        );
    }

    #[test]
    fn mqtt5_metadata_json_preserves_binary_unicode_and_pair_order() {
        let properties = PublishProperties {
            payload_format_indicator: Some(1),
            message_expiry_interval: Some(7),
            content_type: Some("text/é".into()),
            response_topic: Some("reply/\"t".into()),
            correlation_data: Some(vec![0, 255, 7].into()),
            user_properties: vec![
                ("trace".into(), "one".into()),
                ("trace".into(), "two".into()),
            ],
            ..PublishProperties::default()
        };
        assert_eq!(
            render_mqtt5_metadata(&properties).as_deref(),
            Some(
                "{\"payload_format_indicator\":1,\"message_expiry_interval\":7,\"content_type\":\"text/é\",\"response_topic\":\"reply/\\\"t\",\"correlation_data_hex\":\"00ff07\",\"user_properties\":[[\"trace\",\"one\"],[\"trace\",\"two\"]]}"
            )
        );
        assert!(render_mqtt5_metadata(&PublishProperties::default()).is_none());
        assert!(
            render_mqtt5_metadata(&PublishProperties {
                topic_alias: Some(1),
                ..PublishProperties::default()
            })
            .is_none()
        );
    }

    #[test]
    fn mqtt5_metadata_preserves_repeated_subscription_identifiers() {
        let properties = PublishProperties {
            subscription_identifiers: vec![7, 7, 268_435_455],
            ..PublishProperties::default()
        };
        assert_eq!(
            render_mqtt5_metadata(&properties).as_deref(),
            Some("{\"subscription_identifiers\":[7,7,268435455]}")
        );
    }
}
