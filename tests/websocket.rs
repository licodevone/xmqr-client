//! Local WebSocket wire fixture for the opt-in open-lab transport.
use std::{net::TcpListener, process::Command, thread, time::Duration};
use tungstenite::{Message, accept_hdr, handshake::server::Response};

#[expect(
    clippy::result_large_err,
    reason = "tungstenite handshake callback fixes the upstream ErrorResponse type"
)]
fn server(no_protocol: bool, subscribe: bool, mqtt5: bool) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let worker = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut socket = accept_hdr(
            stream,
            |request: &tungstenite::handshake::server::Request, mut response: Response| {
                assert_eq!(request.uri().path(), "/mqtt");
                assert_eq!(
                    request.headers().get("Sec-WebSocket-Protocol").unwrap(),
                    "mqtt"
                );
                if !no_protocol {
                    response
                        .headers_mut()
                        .insert("Sec-WebSocket-Protocol", "mqtt".parse().unwrap());
                }
                Ok(response)
            },
        )
        .unwrap();
        if no_protocol {
            // The client rejects the upgrade before sending MQTT CONNECT.
            return;
        }
        let connect = socket.read().unwrap().into_data();
        assert_eq!(connect[0], 0x10);
        assert!(
            connect
                .windows(5)
                .any(|bytes| bytes == [b'M', b'Q', b'T', b'T', if mqtt5 { 5 } else { 4 }])
        );
        let connack: &[u8] = if mqtt5 {
            &[0x20, 3, 0, 0, 0]
        } else {
            &[0x20, 2, 0, 0]
        };
        socket
            .send(Message::Binary(connack.to_vec().into()))
            .unwrap();
        if subscribe {
            let request = socket.read().unwrap().into_data();
            assert_eq!(request[0], 0x82);
            let id = &request[2..4];
            let suback = if mqtt5 {
                vec![0x90, 4, id[0], id[1], 0, 0]
            } else {
                vec![0x90, 3, id[0], id[1], 0]
            };
            socket.send(Message::Binary(suback.into())).unwrap();
            let mut body = vec![0, 10];
            body.extend_from_slice(b"test/value");
            if mqtt5 {
                body.push(0); // Empty PUBLISH properties.
            }
            body.extend_from_slice(b"hello");
            let mut publish = vec![0x30, u8::try_from(body.len()).unwrap()];
            publish.extend(body);
            socket.send(Message::Binary(publish.into())).unwrap();
        } else {
            let publish = socket.read().unwrap().into_data();
            assert_eq!(publish[0] & 0xf0, 0x30);
            assert!(publish.ends_with(b"ok"));
        }
        for _ in 0..4 {
            let bytes = socket.read().unwrap().into_data();
            if bytes.first() == Some(&0xe0) {
                let _ = socket.close(None);
                return;
            }
            if subscribe && bytes.first() == Some(&0xa2) {
                let id = &bytes[2..4];
                let ack = if mqtt5 {
                    vec![0xb0, 4, id[0], id[1], 0, 0]
                } else {
                    vec![0xb0, 2, id[0], id[1]]
                };
                socket.send(Message::Binary(ack.into())).unwrap();
                continue;
            }
            panic!("unexpected MQTT packet: {bytes:?}");
        }
        panic!("MQTT DISCONNECT missing");
    });
    (port, worker)
}

fn client(port: u16, subscribe: bool, mqtt5: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mqtt-client"));
    command.args([
        if subscribe { "sub" } else { "pub" },
        "--open-lab",
        "--ws-url",
        &format!("ws://127.0.0.1:{port}/mqtt"),
        "--topic",
        "test/value",
        "--client-id",
        "ws-fixture",
        "--timeout-seconds",
        "3",
        "--protocol-version",
        if mqtt5 { "5" } else { "3.1.1" },
    ]);
    if subscribe {
        command.args(["--count", "1"]);
    } else {
        command.args(["--message", "ok"]);
    }
    command.output().unwrap()
}

#[test]
fn websocket_loopback_pub_sub_and_subprotocol_negatives() {
    for mqtt5 in [false, true] {
        for subscribe in [false, true] {
            let (port, worker) = server(false, subscribe, mqtt5);
            let result = client(port, subscribe, mqtt5);
            worker.join().unwrap();
            assert!(
                result.status.success(),
                "mqtt5={mqtt5}, subscribe={subscribe}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            if subscribe {
                assert!(
                    result.stdout.windows(5).any(|window| window == b"hello"),
                    "unexpected stdout: {:?}",
                    String::from_utf8_lossy(&result.stdout)
                );
            }
        }
        let (port, worker) = server(true, false, mqtt5);
        let result = client(port, false, mqtt5);
        worker.join().unwrap();
        assert!(
            !result.status.success(),
            "missing mqtt subprotocol was accepted"
        );
    }
}
