//! Isolated MQTT 3.1.1 fixtures; never run the external broker or use durable state.
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

fn listener() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}

#[cfg(unix)]
#[test]
fn sigterm_sends_disconnect_without_reconnecting() {
    let listener = listener();
    let mut process = child(listener.local_addr().unwrap().port(), "sub", &[]);
    let mut stream = accept(&listener);
    connect(&mut stream, false);
    subscribe(&mut stream, 0);
    // Observe readiness rather than relying on a scheduling sleep.
    let stderr = process.stderr.take().unwrap();
    let mut reader = std::io::BufReader::new(stderr);
    let mut line = String::new();
    std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
    assert!(line.contains("Assinatura ativa"));
    assert!(
        Command::new("kill")
            .args(["-TERM", &process.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(packet(&mut stream), (0xe0, vec![]));
    assert!(finish(process).status.success());
    assert!(listener.accept().is_err());
}

fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                return stream;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "fixture accept deadline");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("fixture accept: {error}"),
        }
    }
}

fn packet(stream: &mut TcpStream) -> (u8, Vec<u8>) {
    let mut first = [0];
    stream.read_exact(&mut first).unwrap();
    let header = first[0];
    let mut length = 0;
    let mut multiplier = 1;
    for _ in 0..4 {
        stream.read_exact(&mut first).unwrap();
        length += usize::from(first[0] & 127) * multiplier;
        if first[0] & 128 == 0 {
            let mut body = vec![0; length];
            stream.read_exact(&mut body).unwrap();
            return (header, body);
        }
        multiplier *= 128;
    }
    panic!("fixture remaining length");
}

fn connect(stream: &mut TcpStream, session_present: bool) -> Vec<u8> {
    let (header, body) = packet(stream);
    assert_eq!(header, 0x10);
    stream
        .write_all(&[0x20, 2, u8::from(session_present), 0])
        .unwrap();
    body
}

fn subscribe(stream: &mut TcpStream, qos: u8) {
    let (header, body) = packet(stream);
    assert_eq!(header, 0x82);
    assert_eq!(body.last(), Some(&qos));
    stream.write_all(&[0x90, 3, body[0], body[1], qos]).unwrap();
}

fn publish(stream: &mut TcpStream) {
    // Retained, QoS0, arbitrary binary payload including invalid UTF-8.
    stream
        .write_all(&[0x31, 7, 0, 1, b't', 0, 255, 10, 34])
        .unwrap();
}

#[test]
fn retained_filters_count_only_displayed_messages_and_ack_filtered_qos() {
    for filter in ["--ignore-retained", "--retained-only"] {
        for qos in 0..=2 {
            let listener = listener();
            let port = listener.local_addr().unwrap().port();
            let only = filter == "--retained-only";
            let server = thread::spawn(move || {
                let mut stream = accept(&listener);
                connect(&mut stream, false);
                subscribe(&mut stream, qos);
                for (identifier, retained, content) in [(7_u8, !only, b'x'), (8, only, b'y')] {
                    let mut body = vec![0, 1, b't'];
                    if qos > 0 {
                        body.extend([0, identifier]);
                    }
                    body.push(content);
                    stream
                        .write_all(
                            &[
                                vec![
                                    0x30 | (qos << 1) | u8::from(retained),
                                    u8::try_from(body.len()).unwrap(),
                                ],
                                body,
                            ]
                            .concat(),
                        )
                        .unwrap();
                    if qos == 1 {
                        assert_eq!(packet(&mut stream), (0x40, vec![0, identifier]));
                    } else if qos == 2 {
                        assert_eq!(packet(&mut stream), (0x50, vec![0, identifier]));
                        stream.write_all(&[0x62, 2, 0, identifier]).unwrap();
                        assert_eq!(packet(&mut stream), (0x70, vec![0, identifier]));
                    }
                }
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            });
            let qos_text = qos.to_string();
            let output = finish(child(
                port,
                "sub",
                &[filter, "--count", "1", "--qos", &qos_text],
            ));
            server.join().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert_eq!(stdout.lines().count(), 1);
            assert!(stdout.contains("\"payload_bytes\":[121]"));
            assert!(stdout.contains(&format!("\"retain\":{only}")));
        }
    }
}

#[test]
fn excluded_topics_still_complete_qos_handshakes_and_count_displayed_only() {
    for qos in 0..=2_u8 {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            subscribe(&mut stream, qos);
            for (identifier, topic, retained) in
                [(7_u8, "t/hidden", true), (8_u8, "t/visible", false)]
            {
                let mut body = vec![0, u8::try_from(topic.len()).unwrap()];
                body.extend(topic.as_bytes());
                if qos > 0 {
                    body.extend([0, identifier]);
                }
                body.push(identifier);
                let header = 0x30 | (qos << 1) | u8::from(retained);
                let mut frame = vec![header, u8::try_from(body.len()).unwrap()];
                frame.extend(body);
                stream.write_all(&frame).unwrap();
                if qos == 1 {
                    assert_eq!(packet(&mut stream), (0x40, vec![0, identifier]));
                } else if qos == 2 {
                    assert_eq!(packet(&mut stream), (0x50, vec![0, identifier]));
                    stream.write_all(&[0x62, 2, 0, identifier]).unwrap();
                    assert_eq!(packet(&mut stream), (0x70, vec![0, identifier]));
                }
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let qos_text = qos.to_string();
        let output = finish(
            Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
                .args([
                    "sub",
                    "--open-lab",
                    "--topic",
                    "t/#",
                    "--port",
                    &port.to_string(),
                    "--client-id",
                    "exclude-id",
                    "--output",
                    "jsonl",
                    "--qos",
                    &qos_text,
                    "--exclude-topic",
                    "t/hidden",
                    "--count",
                    "1",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.lines().count(), 1);
        assert!(stdout.contains("t/visible"));
        assert!(!stdout.contains("t/hidden"));
        assert!(stdout.contains("\"payload_bytes\":[8]"));
    }
}

#[test]
fn raw_and_hex_outputs_preserve_binary_payload() {
    for (format, expected) in [
        ("raw", vec![0, 255, 10, 34]),
        (
            "hex",
            b"topic_hex=74 payload_hex=00ff0a22 qos=0 retain=true\n".to_vec(),
        ),
    ] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            subscribe(&mut stream, 0);
            publish(&mut stream);
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let output = finish(
            Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
                .args([
                    "sub",
                    "--open-lab",
                    "--topic",
                    "t",
                    "--port",
                    &port.to_string(),
                    "--output",
                    format,
                    "--count",
                    "1",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
    }
}

#[test]
fn binary_will_file_is_sent_as_exact_connect_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("will-private-data");
    let expected = [0_u8, 255, 10, 34];
    std::fs::write(&path, expected).unwrap();
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let body = connect(&mut stream, false);
        assert_ne!(body[7] & 0x04, 0);
        let mut cursor = 10;
        for field in [b"stable-id".as_slice(), b"will/topic", expected.as_slice()] {
            let length = usize::from(u16::from_be_bytes([body[cursor], body[cursor + 1]]));
            cursor += 2;
            assert_eq!(&body[cursor..cursor + length], field);
            cursor += length;
        }
        assert_eq!(cursor, body.len());
        assert_eq!(packet(&mut stream).0, 0x30);
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--message",
            "x",
            "--will-topic",
            "will/topic",
            "--will-message-file",
            path.to_str().unwrap(),
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn child(port: u16, command: &str, extra: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
        .args([
            command,
            "--open-lab",
            "--topic",
            "t",
            "--port",
            &port.to_string(),
            "--client-id",
            "stable-id",
            "--output",
            "jsonl",
        ])
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn finish(mut child: Child) -> Output {
    let deadline = Instant::now() + Duration::from_secs(7);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "client deadline: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn mqtt5_publish_qos0_qos1_qos2_and_refusal_wire() {
    for qos in [0_u8, 1, 2] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x10);
            assert_eq!(&body[..11], &[0, 4, b'M', b'Q', b'T', b'T', 5, 2, 0, 30, 0]);
            stream
                .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
                .unwrap();
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x30 | (qos << 1));
            assert_eq!(&body[..3], &[0, 1, b't']);
            let offset = if qos == 0 { 3 } else { 5 };
            assert_eq!(body[offset], 0, "PUBLISH properties must be empty");
            assert_eq!(&body[offset + 1..], b"hello");
            if qos == 1 {
                stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
            } else if qos == 2 {
                stream.write_all(&[0x50, 2, body[3], body[4]]).unwrap();
                assert_eq!(packet(&mut stream), (0x62, vec![body[3], body[4]]));
                stream.write_all(&[0x70, 2, body[3], body[4]]).unwrap();
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let qos_arg = qos.to_string();
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--qos",
                &qos_arg,
                "--message",
                "hello",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0x83, 0]).unwrap();
        let mut probe = [0];
        assert_eq!(stream.read(&mut probe).unwrap(), 0);
    });
    let output = finish(child(
        port,
        "pub",
        &["--protocol-version", "5", "--message", "hello"],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("recusou CONNECT MQTT 5: ImplementationSpecificError (0x83)"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_repeat_waits_for_each_ack_and_reapplies_publish_properties() {
    for qos in [0_u8, 1, 2] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&[0x20, 6, 0, 0, 3, 0x21, 0, 1]).unwrap();
            for _ in 0..3 {
                let (header, body) = packet(&mut stream);
                assert_eq!(header, 0x30 | (qos << 1));
                assert_eq!(&body[..3], &[0, 1, b't']);
                let offset = if qos == 0 { 3 } else { 5 };
                let properties = &body[offset + 1..offset + 1 + usize::from(body[offset])];
                assert_eq!(properties, b"\x01\x01\x26\x00\x05trace\x00\x03one");
                assert_eq!(&body[offset + 1 + properties.len()..], b"hello");
                if qos != 0 {
                    stream
                        .set_read_timeout(Some(Duration::from_millis(30)))
                        .unwrap();
                    let mut probe = [0];
                    assert!(
                        stream.read(&mut probe).is_err(),
                        "next publication preceded ACK"
                    );
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let id = [body[3], body[4]];
                    if qos == 1 {
                        stream.write_all(&[0x40, 2, id[0], id[1]]).unwrap();
                    } else {
                        stream.write_all(&[0x50, 2, id[0], id[1]]).unwrap();
                        assert_eq!(packet(&mut stream), (0x62, id.to_vec()));
                        stream.write_all(&[0x70, 2, id[0], id[1]]).unwrap();
                    }
                }
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let qos_arg = qos.to_string();
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--qos",
                &qos_arg,
                "--message",
                "hello",
                "--repeat",
                "3",
                "--repeat-interval-ms",
                "5",
                "--payload-format",
                "1",
                "--publish-user-property",
                "trace=one",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 3);
    }
}

#[test]
fn mqtt5_repeat_uses_negotiated_topic_alias_and_shortens_subsequent_topics() {
    for qos in [0_u8, 1, 2] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&[0x20, 6, 0, 0, 3, 0x22, 0, 1]).unwrap();
            for index in 0..2 {
                let (header, body) = packet(&mut stream);
                assert_eq!(header, 0x30 | (qos << 1));
                if index == 0 {
                    assert_eq!(&body[..3], &[0, 1, b't']);
                } else {
                    assert_eq!(&body[..2], &[0, 0]);
                }
                let topic_len = if index == 0 { 3 } else { 2 };
                let prop_offset = topic_len + if qos == 0 { 0 } else { 2 };
                let props_len = usize::from(body[prop_offset]);
                let properties = &body[prop_offset + 1..prop_offset + 1 + props_len];
                assert!(properties.windows(3).any(|chunk| chunk == [0x23, 0, 1]));
                assert!(properties.windows(2).any(|chunk| chunk == [1, 1]));
                assert_eq!(&body[prop_offset + 1 + props_len..], b"hello");
                if qos != 0 {
                    let id = [body[topic_len], body[topic_len + 1]];
                    if qos == 1 {
                        stream.write_all(&[0x40, 2, id[0], id[1]]).unwrap();
                    } else {
                        stream.write_all(&[0x50, 2, id[0], id[1]]).unwrap();
                        assert_eq!(packet(&mut stream), (0x62, id.to_vec()));
                        stream.write_all(&[0x70, 2, id[0], id[1]]).unwrap();
                    }
                }
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let qos_arg = qos.to_string();
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--qos",
                &qos_arg,
                "--message",
                "hello",
                "--repeat",
                "2",
                "--repeat-interval-ms",
                "5",
                "--payload-format",
                "1",
                "--publish-topic-alias",
                "1",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 2);
    }
}

#[test]
fn mqtt5_publish_topic_alias_rejects_missing_or_too_small_negotiation_before_publish() {
    for connack in [vec![0x20, 3, 0, 0, 0], vec![0x20, 6, 0, 0, 3, 0x22, 0, 0]] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&connack).unwrap();
            let mut probe = [0];
            assert_eq!(stream.read(&mut probe).unwrap(), 0);
        });
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--message",
                "hello",
                "--repeat",
                "2",
                "--repeat-interval-ms",
                "5",
                "--publish-topic-alias",
                "1",
            ],
        ));
        server.join().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Topic Alias Maximum"));
    }
}

#[test]
fn mqtt5_repeat_stops_on_second_refused_puback_without_third_publish() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        for index in 0..2 {
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x32);
            let id = [body[3], body[4]];
            if index == 0 {
                stream.write_all(&[0x40, 2, id[0], id[1]]).unwrap();
            } else {
                stream.write_all(&[0x40, 4, id[0], id[1], 0x87, 0]).unwrap();
            }
        }
        let mut probe = [0];
        assert_eq!(stream.read(&mut probe).unwrap(), 0);
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--message",
            "hello",
            "--repeat",
            "3",
            "--repeat-interval-ms",
            "5",
        ],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("NotAuthorized (0x87)"));
}

#[test]
fn mqtt5_repeat_timeout_preserves_confirmed_prefix_and_does_not_replay() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x32);
        stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
        assert_eq!(packet(&mut stream).0, 0x32);
        thread::sleep(Duration::from_millis(1300));
        let mut probe = [0];
        assert_eq!(stream.read(&mut probe).unwrap(), 0);
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--message",
            "hello",
            "--repeat",
            "3",
            "--repeat-interval-ms",
            "5",
            "--timeout-seconds",
            "1",
        ],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("resultado desconhecido"));
}

#[test]
fn mqtt5_publish_message_expiry_property_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x30);
        assert_eq!(&body[..3], &[0, 1, b't']);
        assert_eq!(&body[3..9], &[5, 2, 0, 0, 0, 7]);
        assert_eq!(&body[9..], b"hello");
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--message",
            "hello",
            "--message-expiry-seconds",
            "7",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_publish_format_content_type_and_expiry_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x30);
        assert_eq!(&body[..3], &[0, 1, b't']);
        assert_eq!(body[3], 20);
        let properties = &body[4..24];
        assert!(properties.windows(2).any(|part| part == [0x01, 1]));
        assert!(properties.windows(5).any(|part| part == [0x02, 0, 0, 0, 7]));
        assert!(
            properties
                .windows(13)
                .any(|part| part == b"\x03\x00\x0atext/plain")
        );
        assert_eq!(&body[24..], b"hello");
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--message",
            "hello",
            "--message-expiry-seconds",
            "7",
            "--payload-format",
            "1",
            "--content-type",
            "text/plain",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_publish_request_properties_wire_preserves_order() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x30);
        assert_eq!(&body[..3], &[0, 1, b't']);
        let properties = &body[4..4 + usize::from(body[3])];
        assert_eq!(properties, b"\x08\x00\x07reply/t\x09\x00\x03\x00\xff\x07\x26\x00\x05trace\x00\x03one\x26\x00\x05trace\x00\x03two");
        assert_eq!(&body[4 + properties.len()..], b"hello");
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--message",
            "hello",
            "--response-topic",
            "reply/t",
            "--correlation-data-hex",
            "00ff07",
            "--publish-user-property",
            "trace=one",
            "--publish-user-property",
            "trace=two",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_payload_format_one_rejects_binary_before_network() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("binary.bin");
    std::fs::write(&path, [0xff]).unwrap();
    let listener = listener();
    let output = finish(child(
        listener.local_addr().unwrap().port(),
        "pub",
        &[
            "--protocol-version",
            "5",
            "--message-file",
            path.to_str().unwrap(),
            "--payload-format",
            "1",
        ],
    ));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("payload UTF-8 valido"));
    assert!(listener.accept().is_err());
}

#[test]
fn mqtt5_connect_will_delay_property_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(body[7] & 0x06, 0x06);
        let expected = [
            5, 0x18, 0, 0, 0, 2, 0, 4, b'l', b'a', b's', b't', 0, 3, b'b', b'y', b'e',
        ];
        assert!(
            body.windows(expected.len()).any(|part| part == expected),
            "{body:?}"
        );
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream).0, 0x30);
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--message",
            "hello",
            "--will-topic",
            "last",
            "--will-message",
            "bye",
            "--will-delay-seconds",
            "2",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_connect_informational_properties_have_versioned_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(&body[..10], &[0, 4, b'M', b'Q', b'T', b'T', 5, 2, 0, 30]);
        assert_eq!(body[10], 18);
        assert_eq!(
            &body[11..29],
            &[
                0x19, 1, 0x17, 0, 0x26, 0, 1, b'a', 0, 1, b'b', 0x26, 0, 1, b'a', 0, 1, b'c',
            ]
        );
        assert_eq!(
            &body[29..],
            &[0, 9, b's', b't', b'a', b'b', b'l', b'e', b'-', b'i', b'd']
        );
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, publish) = packet(&mut stream);
        assert_eq!(header, 0x32);
        stream
            .write_all(&[0x40, 2, publish[3], publish[4]])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--message",
            "x",
            "--request-response-info",
            "1",
            "--request-problem-info",
            "0",
            "--connect-user-property",
            "a=b",
            "--connect-user-property",
            "a=c",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_persistent_connect_sends_clean_start_and_expiry() {
    for (expiry, value) in [("never", u32::MAX), ("7", 7)] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x10);
            assert_eq!(&body[..10], &[0, 4, b'M', b'Q', b'T', b'T', 5, 0, 0, 30]);
            assert_eq!(body[10], 5);
            assert_eq!(body[11], 0x11);
            assert_eq!(&body[12..16], &value.to_be_bytes());
            stream
                .write_all(&[0x20, 8, 1, 0, 5, 0x27, 0, 1, 0, 0])
                .unwrap();
            let (header, publish) = packet(&mut stream);
            assert_eq!(header, 0x32);
            stream
                .write_all(&[0x40, 2, publish[3], publish[4]])
                .unwrap();
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--clean-start",
                "false",
                "--session-expiry-seconds",
                expiry,
                "--qos",
                "1",
                "--message",
                "x",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn mqtt5_clean_start_rejects_false_session_present() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(body[7] & 2, 2);
        stream.write_all(&[0x20, 3, 1, 0, 0]).unwrap();
    });
    let output = finish(child(
        port,
        "pub",
        &["--protocol-version", "5", "--message", "x"],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("sessao anterior"));
}

#[test]
fn mqtt5_puback_not_authorized_is_reported_without_success_output() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream
            .write_all(&[0x20, 10, 0, 0, 7, 0x24, 1, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x32);
        let packet_id = &body[3..5];
        stream
            .write_all(&[0x40, 4, packet_id[0], packet_id[1], 0x87, 0])
            .unwrap();
        let mut probe = [0];
        assert_eq!(stream.read(&mut probe).unwrap(), 0);
    });
    let output = finish(child(
        port,
        "pub",
        &[
            "--protocol-version",
            "5",
            "--qos",
            "1",
            "--message",
            "hello",
        ],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, [] as [u8; 0]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NotAuthorized (0x87)"), "{stderr}");
    assert!(!stderr.contains("hello"));
}

#[test]
fn mqtt5_qos2_refused_pubrec_or_pubcomp_never_reports_success() {
    for refuse_at_pubrec in [true, false] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream
                .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
                .unwrap();
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x34);
            let id = [body[3], body[4]];
            if refuse_at_pubrec {
                stream.write_all(&[0x50, 4, id[0], id[1], 0x87, 0]).unwrap();
            } else {
                stream.write_all(&[0x50, 2, id[0], id[1]]).unwrap();
                assert_eq!(packet(&mut stream), (0x62, id.to_vec()));
                stream.write_all(&[0x70, 4, id[0], id[1], 0x92, 0]).unwrap();
            }
            let mut probe = [0];
            assert_eq!(stream.read(&mut probe).unwrap(), 0);
        });
        let output = finish(child(
            port,
            "pub",
            &[
                "--protocol-version",
                "5",
                "--qos",
                "2",
                "--message",
                "hello",
            ],
        ));
        server.join().unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, [] as [u8; 0]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = if refuse_at_pubrec {
            "PUBREC: NotAuthorized (0x87)"
        } else {
            "PUBCOMP: PacketIdentifierNotFound (0x92)"
        };
        assert!(stderr.contains(expected), "{stderr}");
    }
}

#[test]
fn mqtt5_subscribe_checks_suback_and_acknowledges_qos1_before_disconnect() {
    for reason in [1_u8, 0, 0x80, 0x87] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream
                .write_all(&[0x20, 10, 0, 0, 7, 0x24, 1, 0x27, 0, 1, 0, 0])
                .unwrap();
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x82);
            assert_eq!(&body[2..], &[0, 0, 1, b't', 1]);
            stream
                .write_all(&[0x90, 4, body[0], body[1], 0, reason])
                .unwrap();
            if reason == 1 {
                stream
                    .write_all(&[0x32, 8, 0, 1, b't', 0, 2, 0, b'h', b'i'])
                    .unwrap();
                assert_eq!(packet(&mut stream), (0x40, vec![0, 2]));
                let (header, body) = packet(&mut stream);
                assert_eq!(header, 0xa2);
                assert_eq!(&body[2..], &[0, 0, 1, b't']);
                stream
                    .write_all(&[0xb0, 4, body[0], body[1], 0, 0])
                    .unwrap();
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            } else {
                let mut probe = [0];
                let _ = stream.read(&mut probe);
            }
        });
        let output = finish(child(
            port,
            "sub",
            &["--protocol-version", "5", "--qos", "1", "--count", "1"],
        ));
        server.join().unwrap();
        if reason == 1 {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("104,105"));
        } else {
            assert!(!output.status.success());
            let stderr = String::from_utf8_lossy(&output.stderr);
            let expected = if reason == 0 {
                "SUBACK MQTT 5 alterou QoS do filtro 1"
            } else {
                "SUBACK MQTT 5 recusou filtro 1"
            };
            assert!(stderr.contains(expected), "{stderr}");
            assert!(stderr.contains(&format!("0x{reason:02X}")), "{stderr}");
        }
    }
}

#[test]
fn mqtt5_subscribe_jsonl_exposes_publish_metadata_without_losing_qos_ack() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 1])
            .unwrap();
        let properties = b"\x01\x01\x02\x00\x00\x00\x07\x03\x00\x0atext/plain\x08\x00\x07reply/t\x09\x00\x03\x00\xff\x07\x26\x00\x05trace\x00\x03one\x26\x00\x05trace\x00\x03two";
        let mut body = vec![0, 1, b't', 0, 2, u8::try_from(properties.len()).unwrap()];
        body.extend_from_slice(properties);
        body.push(b'x');
        stream
            .write_all(&[0x32, u8::try_from(body.len()).unwrap()])
            .unwrap();
        stream.write_all(&body).unwrap();
        assert_eq!(packet(&mut stream), (0x40, vec![0, 2]));
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &["--protocol-version", "5", "--qos", "1", "--count", "1"],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let line = String::from_utf8(output.stdout).unwrap();
    assert!(line.contains("\"payload_bytes\":[120]"), "{line}");
    assert!(line.contains("\"metadata\":{\"payload_format_indicator\":1,\"message_expiry_interval\":7,\"content_type\":\"text/plain\",\"response_topic\":\"reply/t\",\"correlation_data_hex\":\"00ff07\",\"user_properties\":[[\"trace\",\"one\"],[\"trace\",\"two\"]]}"), "{line}");
}

#[test]
fn mqtt5_subscribe_receive_maximum_connect_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(&body[10..14], &[3, 0x21, 0, 1]);
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream.write_all(&[0x30, 5, 0, 1, b't', 0, b'x']).unwrap();
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--receive-maximum",
            "1",
            "--count",
            "1",
        ],
    ));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
}

#[test]
fn mqtt5_subscribe_retain_handling_bits_are_sent_per_filter() {
    for handling in 0_u8..=2 {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
            let (header, subscribe) = packet(&mut stream);
            assert_eq!(header, 0x82);
            assert_eq!(*subscribe.last().unwrap(), handling << 4);
            stream
                .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
                .unwrap();
            stream.write_all(&[0x30, 5, 0, 1, b't', 0, b'x']).unwrap();
            let (header, unsubscribe) = packet(&mut stream);
            assert_eq!(header, 0xa2);
            stream
                .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
                .unwrap();
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let handling_arg = handling.to_string();
        let output = finish(child(
            port,
            "sub",
            &[
                "--protocol-version",
                "5",
                "--retain-handling",
                &handling_arg,
                "--count",
                "1",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn mqtt5_subscribe_no_local_and_retain_as_published_bits_are_independent() {
    for (flags, expected) in [
        (vec!["--no-local"], 0x04),
        (vec!["--retain-as-published"], 0x08),
        (
            vec![
                "--no-local",
                "--retain-as-published",
                "--retain-handling",
                "2",
            ],
            0x2c,
        ),
    ] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
            let (header, subscribe) = packet(&mut stream);
            assert_eq!(header, 0x82);
            assert_eq!(*subscribe.last().unwrap(), expected);
            stream
                .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
                .unwrap();
            stream.write_all(&[0x30, 5, 0, 1, b't', 0, b'x']).unwrap();
            let (header, unsubscribe) = packet(&mut stream);
            assert_eq!(header, 0xa2);
            stream
                .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
                .unwrap();
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let mut args = vec!["--protocol-version", "5", "--count", "1"];
        args.extend(flags);
        let output = finish(child(port, "sub", &args));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn mqtt5_subscription_identifier_is_sent_and_received_in_jsonl() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        assert_eq!(&subscribe[2..5], &[2, 0x0b, 7]);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream
            .write_all(&[0x30, 9, 0, 1, b't', 4, 0x0b, 7, 0x0b, 7, b'x'])
            .unwrap();
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--subscription-identifier",
            "7",
            "--count",
            "1",
        ],
    ));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"subscription_identifiers\":[7,7]"));
}

#[test]
fn mqtt5_subscription_identifier_is_refused_when_connack_disables_it() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 5, 0, 0, 2, 0x29, 0]).unwrap();
        let mut probe = [0];
        assert_eq!(
            stream.read(&mut probe).unwrap(),
            0,
            "client sent SUBSCRIBE despite CONNACK refusal"
        );
    });
    let output = finish(child(
        port,
        "sub",
        &["--protocol-version", "5", "--subscription-identifier", "7"],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Subscription Identifier"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_shared_subscription_filter_is_sent_and_receives_publication() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        assert!(
            subscribe
                .windows(b"$share/workers/t".len())
                .any(|item| item == b"$share/workers/t")
        );
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream.write_all(&[0x30, 5, 0, 1, b't', 0, b'x']).unwrap();
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        assert!(
            unsubscribe
                .windows(b"$share/workers/t".len())
                .any(|item| item == b"$share/workers/t")
        );
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(
        Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
            .args([
                "sub",
                "--open-lab",
                "--protocol-version",
                "5",
                "--topic",
                "$share/workers/t",
                "--port",
                &port.to_string(),
                "--count",
                "1",
                "--output",
                "jsonl",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"topic\":\"t\""));
}

#[test]
fn mqtt5_shared_subscription_is_refused_when_connack_disables_it() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 5, 0, 0, 2, 0x2a, 0]).unwrap();
        let mut probe = [0];
        assert_eq!(
            stream.read(&mut probe).unwrap(),
            0,
            "client sent SUBSCRIBE despite CONNACK refusal"
        );
    });
    let output = finish(
        Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
            .args([
                "sub",
                "--open-lab",
                "--protocol-version",
                "5",
                "--topic",
                "$share/workers/t",
                "--port",
                &port.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("shared subscriptions"));
}

#[test]
fn mqtt5_publish_with_zero_subscription_identifier_is_rejected() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream
            .write_all(&[0x30, 7, 0, 1, b't', 2, 0x0b, 0, b'x'])
            .unwrap();
        let mut probe = [0];
        let _ = stream.read(&mut probe);
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--subscription-identifier",
            "7",
            "--count",
            "1",
        ],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Subscription Identifier"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_subscribe_maximum_packet_size_connect_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(&body[10..16], &[5, 0x27, 0, 0, 0, 100]);
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream.write_all(&[0x30, 5, 0, 1, b't', 0, b'x']).unwrap();
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--maximum-packet-size",
            "100",
            "--count",
            "1",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mqtt5_subscribe_topic_alias_maximum_connect_and_expansion_wire() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x10);
        assert_eq!(&body[10..14], &[3, 0x22, 0, 1]);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream
            .write_all(&[0x30, 8, 0, 1, b't', 3, 0x23, 0, 1, b'x'])
            .unwrap();
        stream
            .write_all(&[0x30, 7, 0, 0, 3, 0x23, 0, 1, b'y'])
            .unwrap();
        let (header, unsubscribe) = packet(&mut stream);
        assert_eq!(header, 0xa2);
        stream
            .write_all(&[0xb0, 4, unsubscribe[0], unsubscribe[1], 0, 0])
            .unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--topic-alias-maximum",
            "1",
            "--count",
            "2",
        ],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8(output.stdout).unwrap();
    assert_eq!(lines.lines().count(), 2);
    assert!(
        lines.lines().all(|line| line.contains("\"topic\":\"t\"")),
        "{lines}"
    );
    assert!(lines.contains("\"payload_bytes\":[120]"), "{lines}");
    assert!(lines.contains("\"payload_bytes\":[121]"), "{lines}");
}

#[test]
fn mqtt5_subscribe_rejects_unestablished_topic_alias_on_new_connection() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        let (header, _) = packet(&mut stream);
        assert_eq!(header, 0x10);
        stream.write_all(&[0x20, 3, 0, 0, 0]).unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 0])
            .unwrap();
        stream
            .write_all(&[0x30, 7, 0, 0, 3, 0x23, 0, 1, b'x'])
            .unwrap();
    });
    let output = finish(child(
        port,
        "sub",
        &[
            "--protocol-version",
            "5",
            "--topic-alias-maximum",
            "1",
            "--count",
            "1",
        ],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, Vec::<u8>::new());
}

#[test]
fn mqtt5_unsuback_refusal_and_timeout_fail_the_subscription_command() {
    for reason in [Some(0x11_u8), Some(0x87), None] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream
                .write_all(&[0x20, 10, 0, 0, 7, 0x24, 1, 0x27, 0, 1, 0, 0])
                .unwrap();
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x82);
            stream
                .write_all(&[0x90, 4, body[0], body[1], 0, 0])
                .unwrap();
            stream
                .write_all(&[0x30, 6, 0, 1, b't', 0, b'h', b'i'])
                .unwrap();
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0xa2);
            assert_eq!(&body[2..], &[0, 0, 1, b't']);
            if let Some(reason) = reason {
                stream
                    .write_all(&[0xb0, 4, body[0], body[1], 0, reason])
                    .unwrap();
                let mut probe = [0];
                assert_eq!(stream.read(&mut probe).unwrap(), 0);
            } else {
                thread::sleep(Duration::from_millis(1300));
            }
        });
        let output = finish(child(
            port,
            "sub",
            &[
                "--protocol-version",
                "5",
                "--count",
                "1",
                "--timeout-seconds",
                "1",
            ],
        ));
        server.join().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(reason) = reason {
            assert!(
                stderr.contains("UNSUBACK MQTT 5 recusou filtro 1"),
                "{stderr}"
            );
            assert!(stderr.contains(&format!("0x{reason:02X}")), "{stderr}");
        } else {
            assert!(
                stderr.contains("tempo esgotado aguardando UNSUBACK MQTT 5"),
                "{stderr}"
            );
        }
    }
}

#[test]
fn mqtt5_subscriber_qos2_completes_handshake_before_disconnect() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        stream
            .write_all(&[0x20, 8, 0, 0, 5, 0x27, 0, 1, 0, 0])
            .unwrap();
        let (header, subscribe) = packet(&mut stream);
        assert_eq!(header, 0x82);
        assert_eq!(*subscribe.last().unwrap(), 2);
        stream
            .write_all(&[0x90, 4, subscribe[0], subscribe[1], 0, 2])
            .unwrap();
        stream
            .write_all(&[0x34, 8, 0, 1, b't', 0, 9, 0, b'h', b'i'])
            .unwrap();
        let mut pubrec = false;
        let mut pubcomp = false;
        let mut unsuback = false;
        while !(pubrec && pubcomp && unsuback) {
            let (header, body) = packet(&mut stream);
            match header {
                0x50 => {
                    assert_eq!(body, [0, 9]);
                    pubrec = true;
                    stream.write_all(&[0x62, 2, 0, 9]).unwrap();
                }
                0x70 => {
                    assert_eq!(body, [0, 9]);
                    pubcomp = true;
                }
                0xa2 => {
                    assert_eq!(&body[2..], &[0, 0, 1, b't']);
                    unsuback = true;
                    stream
                        .write_all(&[0xb0, 4, body[0], body[1], 0, 0])
                        .unwrap();
                }
                _ => panic!("unexpected MQTT 5 packet {header:#x}"),
            }
        }
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &["--protocol-version", "5", "--qos", "2", "--count", "1"],
    ));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("104,105"));
}

#[test]
fn reconnect_preserves_id_and_resubscribes_only_without_session() {
    for session_present in [false, true] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            let first = connect(&mut stream, false);
            assert_eq!(first[7] & 2, 0, "CleanSession=false");
            assert_eq!(&first[8..10], &[0, 47], "selected keepalive survives retry");
            subscribe(&mut stream, 0);
            // Deliver one message to prove count survives the reconnect.
            publish(&mut stream);
            thread::sleep(Duration::from_millis(40));
            drop(stream);
            let mut stream = accept(&listener);
            let second = connect(&mut stream, session_present);
            assert_eq!(first, second, "CONNECT and explicit ID are stable");
            if !session_present {
                subscribe(&mut stream, 0);
            }
            publish(&mut stream);
            assert_eq!(
                packet(&mut stream),
                (0xe0, vec![]),
                "no extra subscribe with resumed session"
            );
        });
        let output = finish(child(
            port,
            "sub",
            &[
                "--count",
                "2",
                "--clean-session",
                "false",
                "--reconnect-attempts",
                "1",
                "--keepalive-seconds",
                "47",
                "--timeout-seconds",
                "2",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = "{\"event\":\"publish\",\"topic\":\"t\",\"qos\":0,\"retain\":true,\"payload_bytes\":[0,255,10,34]}";
        assert_eq!(stdout.lines().collect::<Vec<_>>(), [expected, expected]);
        assert_retry_delay(&output.stderr, "1/1", 80..=100);
    }
}

#[test]
fn repeated_disconnects_exhaust_process_budget() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        for _ in 0..3 {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            subscribe(&mut stream, 0);
            thread::sleep(Duration::from_millis(20));
        }
        // Wait for exit while listener remains alive, proving no fourth CONNECT.
        thread::sleep(Duration::from_millis(250));
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    });
    let output = finish(child(port, "sub", &["--reconnect-attempts", "2"]));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, [] as [u8; 0]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_retry_delay(&output.stderr, "1/2", 80..=100);
    assert_retry_delay(&output.stderr, "2/2", 160..=200);
    assert!(stderr.contains("limite de reconexao esgotado"));
}

fn assert_retry_delay(stderr: &[u8], attempt: &str, range: std::ops::RangeInclusive<u64>) {
    let stderr = String::from_utf8_lossy(stderr);
    let prefix = format!("Reconexao {attempt} em ");
    let line = stderr
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap();
    let milliseconds = line
        .strip_prefix(&prefix)
        .unwrap()
        .strip_suffix("ms")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!(range.contains(&milliseconds));
}

#[test]
fn failed_publication_is_not_replayed() {
    for qos in ["1", "2"] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            assert_eq!(packet(&mut stream).0 >> 4, 3);
            drop(stream); // ACK never arrives.
            thread::sleep(Duration::from_millis(250));
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
            );
        });
        let output = finish(child(
            port,
            "pub",
            &["--message", "x", "--qos", qos, "--reconnect-attempts", "2"],
        ));
        server.join().unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("resultado desconhecido, nao reenviada")
        );
    }
}

#[test]
fn publish_completion_matches_qos_protocol() {
    for qos in [0_u8, 1, 2] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            let (header, body) = packet(&mut stream);
            assert_eq!(header, 0x30 | (qos << 1));
            assert_eq!(body.last(), Some(&b'x'));
            if qos > 0 {
                let id = [body[3], body[4]];
                assert_ne!(id, [0, 0]);
                let ack = if qos == 1 { 0x40 } else { 0x50 };
                stream.write_all(&[ack, 2, id[0], id[1]]).unwrap();
                if qos == 2 {
                    assert_eq!(packet(&mut stream), (0x62, id.to_vec()));
                    stream.write_all(&[0x70, 2, id[0], id[1]]).unwrap();
                }
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let output = finish(child(
            port,
            "pub",
            &["--message", "x", "--qos", &qos.to_string()],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let confirmation = if qos == 0 {
            "sent"
        } else {
            "broker_protocol_ack"
        };
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            format!(
                "{{\"event\":\"publish_complete\",\"qos\":{qos},\"confirmation\":\"{confirmation}\"}}"
            )
        );
    }
}

#[test]
fn repeated_publish_waits_for_ack_and_interval_without_queuing_batch() {
    for qos in 0..=2_u8 {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            let mut previous = None;
            for _ in 0..3 {
                let (header, body) = packet(&mut stream);
                let now = Instant::now();
                if let Some(before) = previous {
                    assert!(now.duration_since(before) >= Duration::from_millis(30));
                }
                previous = Some(now);
                assert_eq!(header, 0x30 | (qos << 1));
                assert_eq!(body.last(), Some(&b'x'));
                if qos > 0 {
                    let id = [body[3], body[4]];
                    stream
                        .write_all(&[if qos == 1 { 0x40 } else { 0x50 }, 2, id[0], id[1]])
                        .unwrap();
                    if qos == 2 {
                        assert_eq!(packet(&mut stream), (0x62, id.to_vec()));
                        stream.write_all(&[0x70, 2, id[0], id[1]]).unwrap();
                    }
                }
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let output = finish(child(
            port,
            "pub",
            &[
                "--message",
                "x",
                "--qos",
                &qos.to_string(),
                "--repeat",
                "3",
                "--repeat-interval-ms",
                "40",
            ],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 3);
    }
}

#[test]
fn publish_before_suback_obeys_count() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x82);
        publish(&mut stream);
        publish(&mut stream);
        stream.write_all(&[0x90, 3, body[0], body[1], 0]).unwrap();
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(port, "sub", &["--count", "1"]));
    server.join().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
}

#[test]
fn sub_sends_all_repeated_topics_and_checks_each_suback_code() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x82);
        let packet_id = [body[0], body[1]];
        let mut cursor = 2;
        let mut filters = Vec::new();
        while cursor < body.len() {
            let length = usize::from(u16::from_be_bytes([body[cursor], body[cursor + 1]]));
            cursor += 2;
            let filter = String::from_utf8(body[cursor..cursor + length].to_vec()).unwrap();
            cursor += length;
            let qos = body[cursor];
            cursor += 1;
            filters.push((filter, qos));
        }
        assert_eq!(filters, [("t".into(), 0), ("other".into(), 0)]);
        stream
            .write_all(&[0x90, 4, packet_id[0], packet_id[1], 0, 0])
            .unwrap();
        publish(&mut stream);
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(port, "sub", &["--topic", "other", "--count", "1"]));
    server.join().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
}

#[test]
fn multi_topic_subscription_rejects_partial_suback() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        let (header, body) = packet(&mut stream);
        assert_eq!(header, 0x82);
        stream.write_all(&[0x90, 3, body[0], body[1], 0]).unwrap();
        let _ = stream.read(&mut [0; 1]);
    });
    let output = finish(child(port, "sub", &["--topic", "other", "--count", "1"]));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("SUBACK"));
}

#[test]
fn connect_rejection_never_retries_or_leaks_peer_payload() {
    for reply in [
        vec![0x20, 2, 0, 5],
        vec![0x30, 9, 0, 1, b't', b's', b'e', b'c', b'r', b'e', b't'],
    ] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            stream.write_all(&reply).unwrap();
            thread::sleep(Duration::from_millis(250));
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
            );
        });
        let output = finish(child(port, "sub", &["--reconnect-attempts", "2"]));
        server.join().unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, [] as [u8; 0]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("secret"));
        assert!(!stderr.contains("Reconexao"));
    }
}

#[test]
fn subscription_qos_acknowledgements_finish_before_count_disconnect() {
    for qos in [1_u8, 2] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            subscribe(&mut stream, qos);
            stream
                .write_all(&[0x30 | (qos << 1), 6, 0, 1, b't', 0, 7, b'x'])
                .unwrap();
            assert_eq!(
                packet(&mut stream),
                (if qos == 1 { 0x40 } else { 0x50 }, vec![0, 7])
            );
            if qos == 2 {
                stream.write_all(&[0x62, 2, 0, 7]).unwrap();
                assert_eq!(packet(&mut stream), (0x70, vec![0, 7]));
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let output = finish(child(
            port,
            "sub",
            &["--count", "1", "--qos", &qos.to_string()],
        ));
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
    }
}

#[test]
fn default_failure_is_immediate_and_initial_connection_can_retry() {
    for attempts in ["0", "1"] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let retry = attempts == "1";
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
            drop(stream); // EOF before CONNACK, the connection was never established.
            if retry {
                let mut stream = accept(&listener);
                connect(&mut stream, false);
                assert_eq!(packet(&mut stream).0, 0x30);
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            } else {
                thread::sleep(Duration::from_millis(250));
                assert!(
                    matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
                );
            }
        });
        let output = finish(child(
            port,
            "pub",
            &["--message", "x", "--reconnect-attempts", attempts],
        ));
        server.join().unwrap();
        assert_eq!(
            output.status.success(),
            retry,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(!output.stdout.is_empty(), retry);
    }
}

#[test]
fn interrupted_incoming_qos2_handshake_does_not_claim_safe_resume() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        subscribe(&mut stream, 2);
        stream
            .write_all(&[0x34, 6, 0, 1, b't', 0, 7, b'x'])
            .unwrap();
        assert_eq!(packet(&mut stream), (0x50, vec![0, 7]));
        drop(stream);
        thread::sleep(Duration::from_millis(250));
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    });
    let output = finish(child(
        port,
        "sub",
        &["--count", "1", "--qos", "2", "--reconnect-attempts", "2"],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("recepcao QoS2 interrompida"));
}

fn stdin_child(port: u16, extra: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
        .args([
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--port",
            &port.to_string(),
            "--client-id",
            "stable-input",
            "--output",
            "jsonl",
            "--stdin",
        ])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn raw_stdin_and_file_preserve_binary_for_all_qos() {
    for qos in 0..=2 {
        for file_source in [false, true] {
            let listener = listener();
            let port = listener.local_addr().unwrap().port();
            let expected = vec![0xef, 0xbb, 0xbf, 0, 255, 13, 10];
            let server_bytes = expected.clone();
            let server = thread::spawn(move || {
                let mut stream = accept(&listener);
                connect(&mut stream, false);
                let (header, body) = packet(&mut stream);
                assert_eq!(header, 0x31 | (qos << 1));
                assert_eq!(&body[..3], &[0, 1, b't']);
                let start = if qos == 0 { 3 } else { 5 };
                assert_eq!(&body[start..], server_bytes);
                if qos == 1 {
                    stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
                }
                if qos == 2 {
                    stream.write_all(&[0x50, 2, body[3], body[4]]).unwrap();
                    assert_eq!(packet(&mut stream), (0x62, body[3..5].to_vec()));
                    stream.write_all(&[0x70, 2, body[3], body[4]]).unwrap();
                }
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            });
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("binary");
            std::fs::write(&path, &expected).unwrap();
            let qos_arg = qos.to_string();
            let process = if file_source {
                child(
                    port,
                    "pub",
                    &[
                        "--message-file",
                        path.to_str().unwrap(),
                        "--qos",
                        &qos_arg,
                        "--retain",
                        "true",
                    ],
                )
            } else {
                let mut process = stdin_child(port, &["--qos", &qos_arg, "--retain", "true"]);
                process.stdin.take().unwrap().write_all(&expected).unwrap();
                process
            };
            let output = finish(process);
            server.join().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
        }
    }
}

#[test]
fn line_mode_waits_for_each_terminal_ack_and_reports_partial_failure() {
    for partial in [false, true] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            let expected: &[&[u8]] = if partial {
                &[b"first"]
            } else {
                &[b"\xff\0", b"", b"last\r"]
            };
            for bytes in expected {
                let (header, body) = packet(&mut stream);
                assert_eq!(header, 0x32);
                assert_eq!(&body[5..], *bytes);
                stream
                    .set_read_timeout(Some(Duration::from_millis(60)))
                    .unwrap();
                let mut probe = [0];
                assert!(stream.read(&mut probe).is_err(), "next PUBLISH before ACK");
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
            }
            if partial {
                let mut probe = [0];
                assert_eq!(stream.read(&mut probe).unwrap(), 0);
            } else {
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            }
        });
        let mut process = stdin_child(
            port,
            &["--line-mode", "--qos", "1", "--reconnect-attempts", "10"],
        );
        let mut input = process.stdin.take().unwrap();
        if partial {
            input.write_all(b"first\n").unwrap();
            input.write_all(&vec![b'x'; 4097]).unwrap();
        } else {
            input.write_all(b"\xff\0\r\n\nlast\r").unwrap();
        }
        drop(input);
        let output = finish(process);
        server.join().unwrap();
        assert_eq!(output.status.success(), !partial);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().lines().count(),
            if partial { 1 } else { 3 }
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Reconexao"));
    }
}

#[test]
fn empty_lines_and_oversized_first_payload_never_connect() {
    for (bytes, extra, success) in [
        (vec![], vec!["--line-mode"], true),
        (vec![0; 4097], vec![], false),
        (vec![0; 4097], vec!["--line-mode"], false),
    ] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let mut process = stdin_child(port, &extra);
        process.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = finish(process);
        assert_eq!(output.status.success(), success);
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(listener.accept().is_err());
    }
    let output = Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        format!("mqtt-client {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert_eq!(output.stderr, [] as [u8; 0]);
}

#[test]
fn open_stdin_times_out_without_waiting_for_native_worker_shutdown() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let mut process = stdin_child(port, &[]);
    let held_input = process.stdin.take().unwrap();
    let started = Instant::now();
    while process.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(11) {
            process.kill().unwrap();
            panic!("stdin shutdown deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = process.wait_with_output().unwrap();
    drop(held_input);
    assert_eq!(output.status.code(), Some(2));
    assert!(listener.accept().is_err());
    assert!(String::from_utf8_lossy(&output.stderr).contains("tempo esgotado"));
}

#[test]
fn connect_carries_default_and_selected_keepalive_bytes() {
    for (extra, expected) in [
        (&[][..], 30_u16),
        (&["--keepalive-seconds", "65535"][..], 65535),
        (&["--keepalive-seconds", "1"][..], 1),
    ] {
        let listener = listener();
        let mut process = child(listener.local_addr().unwrap().port(), "sub", extra);
        let mut stream = accept(&listener);
        let body = connect(&mut stream, false);
        assert_eq!(u16::from_be_bytes([body[8], body[9]]), expected);
        process.kill().unwrap();
        process.wait().unwrap();
    }
}

#[test]
fn configured_timeout_bounds_silent_protocol_stages_without_replay() {
    for stage in ["connack", "suback", "puback", "pubcomp", "incoming_qos2"] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let mode = if matches!(stage, "puback" | "pubcomp") {
            "pub"
        } else {
            "sub"
        };
        let qos = if matches!(stage, "pubcomp" | "incoming_qos2") {
            "2"
        } else {
            "1"
        };
        let mut extra = vec![
            "--timeout-seconds",
            "1",
            "--qos",
            qos,
            "--reconnect-attempts",
            "1",
        ];
        if mode == "pub" {
            extra.extend(["--message", "fixture"]);
        }
        if stage == "incoming_qos2" {
            extra.extend(["--count", "1"]);
        }
        let process = child(port, mode, &extra);
        let mut stream = accept(&listener);
        if stage == "connack" {
            assert_eq!(packet(&mut stream).0, 0x10);
        } else {
            connect(&mut stream, false);
            let (header, body) = packet(&mut stream);
            if stage == "suback" {
                assert_eq!(header, 0x82);
            }
            if stage == "puback" {
                assert_eq!(header >> 4, 3);
            }
            if stage == "pubcomp" {
                assert_eq!(header >> 4, 3);
                let topic_length = usize::from(u16::from_be_bytes([body[0], body[1]]));
                let id = 2 + topic_length;
                stream
                    .write_all(&[0x50, 2, body[id], body[id + 1]])
                    .unwrap();
                assert_eq!(packet(&mut stream).0, 0x62);
            }
            if stage == "incoming_qos2" {
                assert_eq!(header, 0x82);
                stream.write_all(&[0x90, 3, body[0], body[1], 2]).unwrap();
                stream
                    .write_all(&[0x34, 6, 0, 1, b't', 0, 7, b'x'])
                    .unwrap();
                assert_eq!(packet(&mut stream), (0x50, vec![0, 7]));
            }
        }
        let waiting = Instant::now();
        let output = finish(process);
        let elapsed = waiting.elapsed();
        assert!(!output.status.success(), "stage {stage}");
        assert!(
            elapsed >= Duration::from_millis(700),
            "early timeout {stage}: {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(4),
            "late timeout {stage}: {elapsed:?}"
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("Reconexao"),
            "no retry after timeout or PUBLISH"
        );
        assert!(listener.accept().is_err());
    }
}

#[test]
fn payload_boundary_preserves_all_bytes_for_each_qos() {
    for qos in 0..=2_u8 {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let expected: Vec<u8> = (0..4096)
            .map(|index| u8::try_from(index % 256).unwrap())
            .collect();
        let wanted = expected.clone();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            connect(&mut stream, false);
            let (header, body) = packet(&mut stream);
            assert_eq!((header >> 1) & 3, qos);
            let offset = if qos == 0 { 3 } else { 5 };
            assert_eq!(&body[offset..], &wanted);
            if qos == 1 {
                stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
            } else if qos == 2 {
                stream.write_all(&[0x50, 2, body[3], body[4]]).unwrap();
                assert_eq!(packet(&mut stream), (0x62, body[3..5].to_vec()));
                stream.write_all(&[0x70, 2, body[3], body[4]]).unwrap();
            }
            assert_eq!(packet(&mut stream), (0xe0, vec![]));
        });
        let qos_text = qos.to_string();
        let mut process = stdin_child(port, &["--qos", &qos_text]);
        process.stdin.take().unwrap().write_all(&expected).unwrap();
        let output = finish(process);
        server.join().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("publish_complete"));
    }
}

#[test]
fn incoming_packet_limit_reserves_header_even_with_nonminimal_remaining_length() {
    for (remaining, nonminimal, rejected) in [
        (65531_usize, false, false),
        (65532, false, true),
        (65533, false, true),
        (65531, true, false),
        (65532, true, true),
    ] {
        let listener = listener();
        let process = child(
            listener.local_addr().unwrap().port(),
            "sub",
            &["--timeout-seconds", "1"],
        );
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        assert_eq!(packet(&mut stream).0, 0x82);
        let mut length = remaining;
        let mut header = vec![0x30];
        loop {
            let byte = u8::try_from(length % 128).unwrap();
            length /= 128;
            header.push(byte | if length > 0 { 0x80 } else { 0 });
            if length == 0 {
                break;
            }
        }
        if nonminimal {
            *header.last_mut().unwrap() |= 0x80;
            header.push(0);
        }
        assert_eq!(header.len(), if nonminimal { 5 } else { 4 });
        if !rejected {
            assert!(header.len() + remaining <= 65536);
        }
        stream.write_all(&header).unwrap();
        let started = Instant::now();
        let output = finish(process);
        assert!(!output.status.success());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.stdout.len(), 0);
        if rejected {
            assert!(
                diagnostic.contains("pacote MQTT recebido excede limite local de recepcao"),
                "{diagnostic}"
            );
            assert!(started.elapsed() < Duration::from_millis(700));
        } else {
            assert!(diagnostic.contains("tempo esgotado"), "{diagnostic}");
            assert!(started.elapsed() >= Duration::from_millis(700));
        }
        assert!(!diagnostic.contains("quota"));
    }
}

#[test]
fn refused_suback_reports_policy_limits_without_claiming_a_quota_reason() {
    let listener = listener();
    let process = child(
        listener.local_addr().unwrap().port(),
        "sub",
        &["--reconnect-attempts", "1"],
    );
    let mut stream = accept(&listener);
    connect(&mut stream, false);
    let (header, body) = packet(&mut stream);
    assert_eq!(header, 0x82);
    stream
        .write_all(&[0x90, 3, body[0], body[1], 0x80])
        .unwrap();
    let output = finish(process);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("confira ACL e limites do broker"));
    assert!(diagnostic.contains("nao informa causa especifica"));
    assert!(!diagnostic.contains("Reconexao"));
    assert!(listener.accept().is_err());
}

#[test]
fn total_timeout_bounds_connack_suback_publish_and_application_wait() {
    for stage in ["connack", "suback", "puback", "application"] {
        let listener = listener();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let mut stream = accept(&listener);
            if stage == "connack" {
                assert_eq!(packet(&mut stream).0, 0x10);
            } else {
                connect(&mut stream, false);
                if stage == "application" {
                    subscribe(&mut stream, 0);
                } else {
                    let header = packet(&mut stream).0;
                    assert_eq!(header >> 4, if stage == "suback" { 8 } else { 3 });
                }
            }
            let mut byte = [0];
            assert_eq!(
                stream.read(&mut byte).unwrap(),
                0,
                "deadline drops without reconnect"
            );
            assert!(listener.accept().is_err());
        });
        let mut extra = vec![
            "--timeout-seconds",
            "8",
            "--total-timeout-seconds",
            "1",
            "--reconnect-attempts",
            "10",
        ];
        let command = if stage == "puback" {
            extra.extend(["--message", "deadline-fixture", "--qos", "1"]);
            "pub"
        } else {
            "sub"
        };
        let started = Instant::now();
        let output = finish(child(port, command, &extra));
        let elapsed = started.elapsed();
        server.join().unwrap();
        assert!(!output.status.success());
        assert!(elapsed >= Duration::from_millis(800) && elapsed < Duration::from_secs(3));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("tempo total da operacao esgotado")
        );
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
}

#[test]
fn total_timeout_is_not_renewed_by_messages_or_reconnection_backoff() {
    // Frequent application traffic renews idle but must not renew total.
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        subscribe(&mut stream, 0);
        for _ in 0..20 {
            if stream.write_all(&[0x30, 4, 0, 1, b't', b'a']).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(80));
        }
        assert!(listener.accept().is_err());
    });
    let started = Instant::now();
    let output = finish(child(port, "sub", &["--total-timeout-seconds", "1"]));
    let elapsed = started.elapsed();
    server.join().unwrap();
    assert!(!output.status.success());
    assert!(elapsed < Duration::from_secs(3));
    assert_ne!(output.stdout, [] as [u8; 0]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("tempo total da operacao esgotado"));

    let listener = crate::listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        // Four failures consume base 100+200+400+800; total expires during #4 backoff.
        for _ in 0..4 {
            let mut stream = accept(&listener);
            assert_eq!(packet(&mut stream).0, 0x10);
        }
        thread::sleep(Duration::from_millis(900));
        assert!(
            listener.accept().is_err(),
            "no fifth attempt after total expiry"
        );
    });
    let output = finish(child(
        port,
        "sub",
        &["--total-timeout-seconds", "1", "--reconnect-attempts", "10"],
    ));
    server.join().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Reconexao 4/10"));
    assert!(!stderr.contains("Reconexao 5/10"));
    assert!(stderr.contains("tempo total da operacao esgotado"));
}

#[test]
fn qos1_duplicate_delivery_remains_observable_without_exactly_once_claim() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        subscribe(&mut stream, 1);
        for header in [0x32, 0x3a] {
            stream
                .write_all(&[header, 6, 0, 1, b't', 0, 7, b'x'])
                .unwrap();
            assert_eq!(packet(&mut stream), (0x40, vec![0, 7]));
        }
        assert_eq!(packet(&mut stream), (0xe0, vec![]));
    });
    let output = finish(child(
        port,
        "sub",
        &["--qos", "1", "--count", "2", "--total-timeout-seconds", "3"],
    ));
    server.join().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 2);
}

#[cfg(unix)]
#[test]
fn signals_cancel_connack_poll_backoff_and_following_input_without_retry() {
    for signal in ["-INT", "-TERM"] {
        for stage in ["poll", "backoff", "input"] {
            let listener = listener();
            let port = listener.local_addr().unwrap().port();
            let mut process = if stage == "input" {
                let mut process = stdin_child(
                    port,
                    &["--line-mode", "--qos", "1", "--reconnect-attempts", "10"],
                );
                process
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b"first\n")
                    .unwrap();
                process
            } else {
                child(port, "sub", &["--reconnect-attempts", "10"])
            };
            let mut stream = accept(&listener);
            if stage == "input" {
                connect(&mut stream, false);
                let (_, body) = packet(&mut stream);
                stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
                let stdout = process.stdout.take().unwrap();
                let mut reader = std::io::BufReader::new(stdout);
                let mut line = String::new();
                std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
                assert!(line.contains("publish_complete"));
            } else {
                assert_eq!(packet(&mut stream).0, 0x10);
                if stage == "backoff" {
                    drop(stream);
                    // Observe sixth (>=2560ms) backoff before signal, not a guessed sleep.
                    let stderr = process.stderr.take().unwrap();
                    let mut reader = std::io::BufReader::new(stderr);
                    for attempt in 1..=6 {
                        if attempt > 1 {
                            stream = accept(&listener);
                            assert_eq!(packet(&mut stream).0, 0x10);
                            drop(stream);
                        }
                        let mut line = String::new();
                        std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
                        assert!(line.contains(&format!("Reconexao {attempt}/10")));
                    }
                    assert!(
                        Command::new("kill")
                            .args([signal, &process.id().to_string()])
                            .status()
                            .unwrap()
                            .success()
                    );
                    assert!(finish(process).status.success());
                    thread::sleep(Duration::from_millis(50));
                    assert!(listener.accept().is_err());
                    continue;
                }
            }
            assert!(
                Command::new("kill")
                    .args([signal, &process.id().to_string()])
                    .status()
                    .unwrap()
                    .success()
            );
            if stage == "input" {
                assert_eq!(packet(&mut stream), (0xe0, vec![]));
            } else {
                let mut byte = [0];
                assert_eq!(stream.read(&mut byte).unwrap(), 0);
            }
            assert!(finish(process).status.success());
            assert!(listener.accept().is_err());
        }
    }
}

#[test]
fn total_timeout_also_bounds_following_payload_input_without_new_publish() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let mut process = stdin_child(
        port,
        &["--line-mode", "--qos", "1", "--total-timeout-seconds", "1"],
    );
    process
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"first\n")
        .unwrap();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        let (_, body) = packet(&mut stream);
        stream.write_all(&[0x40, 2, body[3], body[4]]).unwrap();
        let mut byte = [0];
        assert_eq!(
            stream.read(&mut byte).unwrap(),
            0,
            "no second PUBLISH nor retry after total expiry"
        );
        assert!(listener.accept().is_err());
    });
    let started = Instant::now();
    let output = finish(process);
    server.join().unwrap();
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("tempo total da operacao esgotado"));
}

#[test]
fn reconnect_connack_is_protocol_bounded_not_ninety_second_idle() {
    let listener = listener();
    let port = listener.local_addr().unwrap().port();
    let (ready, resume) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let mut stream = accept(&listener);
        connect(&mut stream, false);
        subscribe(&mut stream, 0);
        resume.recv_timeout(Duration::from_secs(3)).unwrap();
        drop(stream);
        let mut stream = accept(&listener);
        assert_eq!(packet(&mut stream).0, 0x10);
        // A silent reconnected peer cannot borrow the application's idle90s.
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte).unwrap(), 0);
        assert!(listener.accept().is_err());
    });
    let started = Instant::now();
    let mut process = child(
        port,
        "sub",
        &["--timeout-seconds", "1", "--reconnect-attempts", "10"],
    );
    let mut reader = std::io::BufReader::new(process.stderr.take().unwrap());
    let mut line = String::new();
    std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
    assert!(line.contains("Assinatura ativa"));
    process.stderr = Some(reader.into_inner());
    ready.send(()).unwrap();
    let output = finish(process);
    server.join().unwrap();
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tempo esgotado aguardando resposta do broker"));
    assert!(stderr.contains("Reconexao 1/10"));
    assert!(!stderr.contains("Reconexao 2/10"));
}

#[test]
fn invalid_total_timeout_is_static_and_never_opens_network() {
    let listener = listener();
    for value in ["0", "86401", "private-fixture-input", "4294967296"] {
        let output = finish(child(
            listener.local_addr().unwrap().port(),
            "sub",
            &["--total-timeout-seconds", value],
        ));
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("--total-timeout-seconds deve ser 1..86400"));
        assert!(!stderr.contains("private-fixture-input"));
        assert!(listener.accept().is_err());
    }
}
