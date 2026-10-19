//! C144: independent wire vectors for the four P232 receiving recommendations.
use bytes::BytesMut;
use rumqttc::v5::mqttbytes::{
    Error,
    v5::{Packet, check},
};

fn vbi(mut value: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = u8::try_from(value % 128).unwrap();
        value /= 128;
        if value > 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            return bytes;
        }
    }
}

fn frame(header: u8, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![header];
    bytes.extend(vbi(body.len()));
    bytes.extend(body);
    bytes
}

fn string(value: &[u8]) -> Vec<u8> {
    let mut bytes = u16::try_from(value.len()).unwrap().to_be_bytes().to_vec();
    bytes.extend(value);
    bytes
}

fn publication(topic: &[u8], properties: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut body = string(topic);
    body.extend(vbi(properties.len()));
    body.extend(properties);
    body.extend(payload);
    frame(0x30, &body)
}

fn decode(bytes: &[u8]) -> Result<Packet, Error> {
    Packet::read(&mut BytesMut::from(bytes), Some(65536))
}

#[test]
fn nul_and_malformed_utf8_topics_are_rejected_before_delivery() {
    for topic in [b"audit/\0topic".as_slice(), b"audit/\xff", b"\xed\xa0\x80"] {
        assert!(
            decode(&publication(topic, &[], b"data")).is_err(),
            "{topic:?}"
        );
    }
}

#[test]
fn user_property_keys_and_values_reject_nul_across_received_packet_types() {
    for (key, value) in [(b"\0".as_slice(), b"v".as_slice()), (b"k", b"v\0")] {
        let mut properties = vec![0x26];
        properties.extend(string(key));
        properties.extend(string(value));
        let mut connack = vec![0, 0];
        connack.extend(vbi(properties.len()));
        connack.extend(&properties);
        let mut puback = vec![0, 1, 0];
        puback.extend(vbi(properties.len()));
        puback.extend(&properties);
        for wire in [
            frame(0x20, &connack),
            frame(0x40, &puback),
            publication(b"audit/t", &properties, b"data"),
        ] {
            assert!(decode(&wire).is_err(), "{wire:?}");
        }
    }
}

#[test]
fn unicode_empty_user_properties_bom_and_binary_data_remain_intact() {
    let mut properties = vec![0x26];
    properties.extend(string(b""));
    properties.extend(string("ação/🤖\u{feff}".as_bytes()));
    properties.push(0x09); // Correlation Data is binary, not an MQTT string.
    properties.extend(string(b"\0\xff\xc0\x80"));
    let payload = b"\0\xff\xc0\x80";
    let topic = "audit/ação/🤖\u{feff}".as_bytes();
    let Packet::Publish(publish) = decode(&publication(topic, &properties, payload)).unwrap()
    else {
        panic!("expected PUBLISH");
    };
    assert_eq!(publish.topic.as_ref(), topic);
    assert_eq!(publish.payload.as_ref(), payload);
    let properties = publish.properties.unwrap();
    assert_eq!(
        properties.user_properties,
        [(String::new(), "ação/🤖\u{feff}".to_owned())]
    );
    assert_eq!(properties.correlation_data.unwrap().as_ref(), payload);
    // Empty topic with Topic Alias is a valid representation.
    assert!(decode(&publication(b"", &[0x23, 0, 1], payload)).is_ok());
}

#[test]
fn vbi_is_minimal_at_every_width_boundary_and_truncated_data_is_not_consumed() {
    for value in [
        0,
        1,
        127,
        128,
        16383,
        16384,
        2_097_151,
        2_097_152,
        268_435_455,
    ] {
        let canonical = vbi(value);
        let mut header = vec![0x30];
        header.extend(&canonical);
        if value == 0 {
            assert!(check(header.iter(), None).is_ok());
        } else {
            assert!(
                matches!(check(header.iter(), None), Err(Error::InsufficientBytes(n)) if n == value)
            );
        }
        for width in canonical.len() + 1..=4 {
            let mut redundant = canonical.clone();
            while redundant.len() < width {
                *redundant.last_mut().unwrap() |= 0x80;
                redundant.push(0);
            }
            let mut wire = vec![0x30];
            wire.extend(redundant);
            assert!(
                matches!(
                    check(wire.iter(), None),
                    Err(Error::MalformedRemainingLength)
                ),
                "{wire:?}"
            );
        }
    }
    for wire in [
        &[0x30, 0x80][..],
        &[0x30, 0x80, 0x80],
        &[0x30, 0x80, 0x80, 0x80],
    ] {
        let mut stream = BytesMut::from(wire);
        assert!(matches!(
            Packet::read(&mut stream, None),
            Err(Error::InsufficientBytes(_))
        ));
        assert_eq!(stream.as_ref(), wire);
    }
    assert!(matches!(
        check([0x30, 0x80, 0x80, 0x80, 0x80].iter(), None),
        Err(Error::MalformedRemainingLength)
    ));
    assert!(matches!(
        check([0x30, 0x80, 0x80, 0x01].iter(), Some(4096)),
        Err(Error::PayloadSizeLimitExceeded { .. })
    ));
}

#[test]
fn redundant_property_lengths_and_subscription_identifiers_are_rejected() {
    assert!(decode(&[0x20, 0x04, 0, 0, 0x80, 0]).is_err());
    assert!(decode(&[0x40, 0x05, 0, 1, 0, 0x80, 0]).is_err());
    assert!(decode(&publication(b"audit/t", &[0x0b, 0x81, 0], b"payload")).is_err());
    for identifier in [1, 127, 128, 16383, 16384, 2_097_151, 2_097_152, 268_435_455] {
        let mut properties = vec![0x0b];
        properties.extend(vbi(identifier));
        let Packet::Publish(publish) =
            decode(&publication(b"audit/t", &properties, b"\0")).unwrap()
        else {
            panic!("expected PUBLISH");
        };
        assert_eq!(
            publish.properties.unwrap().subscription_identifiers,
            [identifier]
        );
        assert_eq!(publish.payload.as_ref(), b"\0");
    }
}

#[test]
fn all_reserved_fixed_header_flags_are_checked_before_waiting_for_a_body() {
    for kind in 1..=15 {
        if kind == 3 {
            continue; // PUBLISH flags carry their own semantics.
        }
        let expected = if [6, 8, 10].contains(&kind) { 2 } else { 0 };
        for flags in 0..=15 {
            let wire = [kind << 4 | flags, 0];
            assert_eq!(
                check(wire.iter(), None).is_ok(),
                flags == expected,
                "{wire:?}"
            );
        }
    }
    assert!(decode(&[0x40, 2, 0, 1]).is_ok());
    assert!(decode(&[0x62, 2, 0, 1]).is_ok());
    assert!(matches!(
        check([0x41, 0xff, 0x7f].iter(), None),
        Err(Error::IncorrectPacketFormat)
    ));
}

#[test]
fn fragmented_and_concatenated_canonical_packets_preserve_payload_and_boundaries() {
    let wire = publication(b"audit/t", &[], &[0; 128]);
    for split in 0..wire.len() {
        let mut stream = BytesMut::from(&wire[..split]);
        assert!(matches!(
            Packet::read(&mut stream, Some(65536)),
            Err(Error::InsufficientBytes(_))
        ));
        assert_eq!(stream.as_ref(), &wire[..split]);
        stream.extend_from_slice(&wire[split..]);
        stream.extend_from_slice(&[0x40, 2, 0, 1]);
        assert!(
            matches!(Packet::read(&mut stream, Some(65536)), Ok(Packet::Publish(p)) if p.payload.as_ref() == [0; 128])
        );
        assert!(matches!(
            Packet::read(&mut stream, Some(65536)),
            Ok(Packet::PubAck(_))
        ));
        assert!(stream.is_empty());
    }
}
