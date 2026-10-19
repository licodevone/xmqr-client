//! C142: exercise the patched dependency through its public API.
use rumqttc::Outgoing;
use rumqttc::v5::{
    Event, Incoming, MqttState,
    mqttbytes::{
        QoS,
        v5::{Packet, PubCompReason, PubRel, PubRelReason, Publish},
    },
};

fn publication(id: u16, duplicate: bool) -> Incoming {
    let mut publication =
        Publish::new("audit/topic", QoS::ExactlyOnce, b"synthetic".to_vec(), None);
    publication.pkid = id;
    publication.dup = duplicate;
    Incoming::Publish(publication)
}

fn application_deliveries(state: &mut MqttState) -> usize {
    state
        .events
        .drain(..)
        .filter(|event| matches!(event, Event::Incoming(Incoming::Publish(_))))
        .count()
}

fn expect_pubcomp(state: &mut MqttState, id: u16, reason: PubCompReason) {
    let response = state
        .handle_incoming_packet(Incoming::PubRel(PubRel::new(id, None)))
        .unwrap();
    assert!(
        matches!(response, Some(Packet::PubComp(ack)) if ack.pkid == id && ack.reason == reason)
    );
}

#[test]
fn qos2_duplicate_acknowledges_without_redelivery_and_identifier_is_reusable() {
    let mut state = MqttState::new(8, false);
    for duplicate in [false, true, false] {
        // Receiver obligation does not depend on the incoming DUP flag.
        let response = state
            .handle_incoming_packet(publication(49, duplicate))
            .unwrap();
        assert!(matches!(response, Some(Packet::PubRec(ack)) if ack.pkid == 49));
    }
    assert_eq!(application_deliveries(&mut state), 1);
    expect_pubcomp(&mut state, 49, PubCompReason::Success);
    expect_pubcomp(&mut state, 49, PubCompReason::PacketIdentifierNotFound);
    assert_eq!(application_deliveries(&mut state), 0);
    state
        .handle_incoming_packet(publication(49, false))
        .unwrap();
    state
        .handle_incoming_packet(publication(50, false))
        .unwrap();
    assert_eq!(application_deliveries(&mut state), 2);
    expect_pubcomp(&mut state, 49, PubCompReason::Success);
    expect_pubcomp(&mut state, 50, PubCompReason::Success);
}

#[test]
fn suppressed_duplicate_still_produces_event_for_eventloop_progress() {
    let mut state = MqttState::new(8, false);
    state
        .handle_incoming_packet(publication(65535, false))
        .unwrap();
    state.events.clear();
    state
        .handle_incoming_packet(publication(65535, true))
        .unwrap();
    assert!(matches!(
        state.events.pop_front(),
        Some(Event::Outgoing(Outgoing::PubRec(65535)))
    ));
    assert!(state.events.is_empty());
}

#[test]
fn unknown_pubrel_gets_92_and_zero_remains_invalid() {
    let mut state = MqttState::new(8, false);
    expect_pubcomp(&mut state, 65535, PubCompReason::PacketIdentifierNotFound);
    assert!(
        state
            .handle_incoming_packet(Incoming::PubRel(PubRel::new(0, None)))
            .is_err()
    );
}

#[test]
fn error_pubrel_still_completes_known_exchange() {
    let mut state = MqttState::new(8, false);
    state.handle_incoming_packet(publication(1, false)).unwrap();
    let mut release = PubRel::new(1, None);
    release.reason = PubRelReason::PacketIdentifierNotFound;
    assert!(
        matches!(state.handle_incoming_packet(Incoming::PubRel(release)).unwrap(),
        Some(Packet::PubComp(ack)) if ack.reason == PubCompReason::Success)
    );
    expect_pubcomp(&mut state, 1, PubCompReason::PacketIdentifierNotFound);
}

#[test]
fn durable_manual_receiver_keeps_duplicate_events_and_controls_pubcomp() {
    let mut state = MqttState::new(8, true);
    state.set_manual_incoming_qos2(true);
    for duplicate in [false, true] {
        assert!(
            state
                .handle_incoming_packet(publication(49, duplicate))
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(application_deliveries(&mut state), 2);
    for id in [49, 50] {
        assert!(
            state
                .handle_incoming_packet(Incoming::PubRel(PubRel::new(id, None)))
                .unwrap()
                .is_none()
        );
        assert!(
            matches!(state.events.pop_front(), Some(Event::Incoming(Incoming::PubRel(ack))) if ack.pkid == id)
        );
        assert!(state.events.is_empty());
    }
}
