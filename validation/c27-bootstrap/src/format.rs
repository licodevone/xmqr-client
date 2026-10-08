// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
// C27 contract reconstruction: newly authored implementation, not production source.
use rumqttc::{QoS, mqttbytes::v4::Publish};
use std::fmt::Write;

pub const fn level(qos: QoS) -> u8 {
    match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    }
}

fn quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        if c == '"' {
            out.push_str("\\\"");
        } else if c == '\\' {
            out.push_str("\\\\");
        } else if c <= '\u{1f}' {
            write!(out, "\\u{:04x}", u32::from(c)).unwrap();
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}

pub fn received(json: bool, p: &Publish) -> String {
    if !json {
        return format!(
            "topico={:?} qos={} retain={} mensagem={:?}",
            p.topic,
            level(p.qos),
            p.retain,
            String::from_utf8_lossy(&p.payload)
        );
    }
    let mut bytes = String::new();
    for (index, byte) in p.payload.iter().enumerate() {
        if index != 0 {
            bytes.push(',');
        }
        write!(bytes, "{byte}").unwrap();
    }
    format!(
        "{{\"event\":\"publish\",\"topic\":{},\"qos\":{},\"retain\":{},\"payload_bytes\":[{bytes}]}}",
        quoted(&p.topic),
        level(p.qos),
        p.retain
    )
}

pub fn completed(json: bool, qos: QoS) -> String {
    if json {
        format!(
            "{{\"event\":\"publish_complete\",\"qos\":{},\"confirmation\":\"{}\"}}",
            level(qos),
            if qos == QoS::AtMostOnce {
                "sent"
            } else {
                "broker_protocol_ack"
            }
        )
    } else if qos == QoS::AtMostOnce {
        "PUBLISH QoS 0 enviado; o protocolo nao confirma entrega ao assinante.".into()
    } else {
        format!("Publicacao confirmada pelo broker em QoS {}.", level(qos))
    }
}

pub fn active(json: bool) {
    let text = "Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.";
    if json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_preserves_every_byte_and_all_controls() {
        let payload: Vec<u8> = (0..=255).collect();
        let p = Publish::new("t", QoS::AtMostOnce, payload.clone());
        let expected = payload
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        assert!(received(true, &p).contains(&format!("\"payload_bytes\":[{expected}]")));
        let controls: String = (0..=31).map(|n| char::from_u32(n).unwrap()).collect();
        let encoded = quoted(&controls);
        assert!(encoded.contains("\\u0000"));
        assert!(encoded.contains("\\u001f"));
        assert_eq!(quoted("é\"\\"), "\"é\\\"\\\\\"");
        assert!(!encoded.contains('\n'));
    }
    #[test]
    fn text_empty_and_completion_contracts() {
        let p = Publish::new("t", QoS::AtMostOnce, vec![]);
        assert_eq!(
            received(false, &p),
            "topico=\"t\" qos=0 retain=false mensagem=\"\""
        );
        assert!(received(true, &p).contains("\"payload_bytes\":[]"));
        assert!(completed(true, QoS::AtLeastOnce).contains("broker_protocol_ack"));
        assert!(completed(true, QoS::AtMostOnce).contains("sent"));
    }
}
