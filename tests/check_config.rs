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
