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
        assert!(String::from_utf8_lossy(&output.stderr).contains("Reconexao 1/1 em 100ms"));
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
    assert!(stderr.contains("Reconexao 1/2 em 100ms"));
    assert!(stderr.contains("Reconexao 2/2 em 200ms"));
    assert!(stderr.contains("limite de reconexao esgotado"));
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
