use std::process::{Command, Stdio};

fn check() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mqtt-client"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("MQTT_") {
            command.env_remove(key);
        }
    }
    command.arg("--check-config").stdin(Stdio::null());
    command
}

#[test]
fn bench_rejects_unbounded_load_and_unsafe_topic_prefixes() {
    for args in [
        vec!["conn", "--clients", "1001"],
        vec!["conn", "--hold-seconds", "0"],
        vec!["conn", "--host", "--clients", "2", "127.0.0.1"],
        vec!["reconnect", "--messages", "1"],
        vec!["echo", "--topic-prefix", "private-fixture/#"],
        vec!["echo", "--messages", "101", "--topic-prefix", "fixture"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
            .args(["bench", args[0], "--open-lab"])
            .args(&args[1..])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["operation"], "bench");
        assert_eq!(report["reason"], "invalid_options");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private-fixture"));
    }
}

#[test]
fn probe_rejects_unsafe_or_unknown_options_without_echoing_values() {
    for options in [
        vec!["--round-trip"],
        vec!["--topic", "private-fixture/#"],
        vec!["--host", "192.0.2.1"],
        vec!["--client-id", "private-fixture"],
        vec!["--will-topic", "private-fixture"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mqtt-client"))
            .args(["probe", "--open-lab"])
            .args(options)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["phase"], "configuration");
        assert_eq!(report["reason"], "invalid_options");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private-fixture"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn validates_pub_stdin_without_reading_payload_and_sub_mqtt5() {
    let output = check()
        .args(["pub", "--open-lab", "--topic", "fixture", "--stdin"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains(r#""operation":"pub""#));
    assert!(json.contains(r#""password_source":"none""#));

    let output = check()
        .args([
            "sub",
            "--open-lab",
            "--topic",
            "fixture/#",
            "--protocol-version",
            "5",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains(r#""operation":"sub""#));
    assert!(json.contains(r#""protocol":"5""#));
}

#[test]
fn checks_tls_material_referenced_by_profile() {
    let directory = tempfile::tempdir().unwrap();
    let profile = directory.path().join("client.toml");
    std::fs::write(
        &profile,
        "version=1\n[connection]\ntls_password=true\nusername='device'\nca='private-fixture'\n",
    )
    .unwrap();
    let output = check()
        .args(["sub", "--topic", "fixture", "--profile"])
        .arg(profile)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("materiais TLS"));
    assert!(!error.contains("private-fixture"));
}

#[test]
fn redacts_invalid_arguments_and_missing_private_files() {
    for args in [
        vec!["private-fixture"],
        vec![
            "sub",
            "--plain-auth-lab",
            "--topic",
            "fixture",
            "--username",
            "device",
            "--password-file",
            "private-fixture",
        ],
        vec![
            "sub",
            "--tls-password",
            "--topic",
            "fixture",
            "--username",
            "device",
            "--ca",
            "private-fixture",
        ],
    ] {
        let output = check().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-fixture"));
    }
}
