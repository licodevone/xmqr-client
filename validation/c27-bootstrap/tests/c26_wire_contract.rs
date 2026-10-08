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
    assert!(output.stdout.is_empty());
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
        assert!(output.stdout.is_empty());
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
        assert!(output.stdout.is_empty());
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
