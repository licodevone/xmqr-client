//! Bounded, opt-in connection profiles. Never accept inline credentials or payloads.
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    version: u32,
    connection: BTreeMap<String, toml::Value>,
}

pub fn expand(args: Vec<String>) -> Result<Vec<String>, String> {
    let mut positions = Vec::new();
    let mut explicit = std::collections::BTreeSet::new();
    let mut cursor = 1;
    while cursor < args.len() {
        let flag = &args[cursor];
        explicit.insert(flag.clone());
        if flag == "--profile" {
            positions.push(cursor);
        }
        cursor += if matches!(
            flag.as_str(),
            "--stdin"
                | "--line-mode"
                | "--ignore-retained"
                | "--retained-only"
                | "--open-lab"
                | "--plain-auth-lab"
                | "--tls-password"
        ) {
            1
        } else {
            2
        };
    }
    if positions.is_empty() {
        return Ok(args);
    }
    if positions.len() != 1 || positions[0] == 0 {
        return Err("use --profile uma unica vez apos pub/sub".into());
    }
    if explicit.contains("--open-lab") || explicit.contains("--plain-auth-lab") {
        return Err("perfil de conexao exige mTLS".into());
    }
    let index = positions[0];
    let path = args
        .get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .ok_or("informe o arquivo de perfil")?;
    let path = Path::new(path);
    if !path
        .metadata()
        .map_err(|_| "perfil indisponivel")?
        .is_file()
    {
        return Err("perfil exige arquivo regular".into());
    }
    let mut file = File::open(path).map_err(|_| "nao foi possivel abrir perfil")?;
    if !file
        .metadata()
        .map_err(|_| "perfil indisponivel")?
        .is_file()
    {
        return Err("perfil exige arquivo regular".into());
    }
    let mut data = String::new();
    file.by_ref()
        .take(65537)
        .read_to_string(&mut data)
        .map_err(|_| "perfil invalido")?;
    if data.len() > 65536 {
        return Err("perfil excede 64KiB".into());
    }
    let profile: Profile = toml::from_str(&data).map_err(|_| "schema de perfil invalido")?;
    if profile.version != 1 {
        return Err("versao de perfil invalida".into());
    }
    let mut expanded = vec![args[0].clone()];
    for (key, value) in profile.connection {
        if key == "tls_password" {
            if value.as_bool() != Some(true) {
                return Err("tls_password exige true".into());
            }
            if !explicit.contains("--tls-password") {
                expanded.push("--tls-password".into());
            }
            continue;
        }
        let (flag, string) = setting(&key, &value, path)?;
        // Existing CLI validation checks the effective values after precedence.
        if !explicit.contains(&flag) {
            expanded.extend([flag, string]);
        }
    }
    expanded.extend(
        args.into_iter()
            .enumerate()
            .skip(1)
            .filter(|(position, _)| *position != index && *position != index + 1)
            .map(|(_, value)| value),
    );
    Ok(expanded)
}

fn setting(
    key: &str,
    value: &toml::Value,
    profile_path: &Path,
) -> Result<(String, String), String> {
    let (string, is_path) = match key {
        "host" | "client_id" | "username" | "output" => (value.as_str().map(str::to_owned), false),
        "ca" | "cert" | "key" | "password_file" => (value.as_str().map(str::to_owned), true),
        "port" | "qos" | "keepalive_seconds" | "timeout_seconds" => {
            (value.as_integer().map(|number| number.to_string()), false)
        }
        _ => return Err("campo de perfil nao permitido".into()),
    };
    let mut string = string
        .filter(|value| !value.is_empty())
        .ok_or("tipo ou valor de perfil invalido")?;
    if is_path {
        let location = Path::new(&string);
        if !location.is_absolute() {
            string = profile_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(location)
                .into_os_string()
                .into_string()
                .map_err(|_| "caminho de perfil exige UTF-8")?;
        }
    }
    Ok((format!("--{}", key.replace('_', "-")), string))
}

#[cfg(test)]
mod tests {
    fn parse_profile(text: &str, extra: &[&str]) -> Result<crate::cli::Cli, String> {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("connection.toml");
        std::fs::write(&path, text).unwrap();
        let mut args = vec![
            "sub".to_owned(),
            "--topic".into(),
            "sensor/#".into(),
            "--profile".into(),
            path.to_str().unwrap().into(),
        ];
        args.extend(extra.iter().map(|value| (*value).to_owned()));
        crate::cli::parse(args)
    }
    #[test]
    fn paths_precedence_and_bounded_effective_values() {
        let text = "version=1\n[connection]\nhost='127.0.0.1'\nport=8883\nusername='device'\nca='ca.crt'\ncert='client.crt'\nkey='client.key'\nqos=2\n";
        let cli = parse_profile(text, &["--host", "localhost", "--port", "8884"]).unwrap();
        assert_eq!(cli.host, "localhost");
        assert_eq!(cli.port, 8884);
        assert!(cli.ca.unwrap().is_absolute());
        assert_eq!(cli.qos, rumqttc::QoS::ExactlyOnce);
        assert!(parse_profile(text, &["--host", "a", "--host", "b"]).is_err());
        assert!(parse_profile(text, &["--profile", "other.toml"]).is_err());
        assert!(parse_profile(text, &["--open-lab"]).is_err());
    }

    #[test]
    fn tls_password_profile_selects_no_client_certificate() {
        let text = "version=1\n[connection]\ntls_password=true\nusername='device'\nca='ca.crt'\n";
        let cli = parse_profile(text, &[]).unwrap();
        assert_eq!(cli.transport_mode, crate::cli::TransportMode::TlsPassword);
        assert!(cli.cert.is_none() && cli.key.is_none());
        assert!(parse_profile(&text.replace("true", "false"), &[]).is_err());
        assert!(parse_profile(text, &["--plain-auth-lab"]).is_err());
    }
    #[test]
    fn unknown_secrets_schema_and_size_never_leak_contents() {
        for text in [
            "version=2\n[connection]",
            "version=1\n[connection]\npassword='private-sentinel'",
            "version=1\n[connection]\nport='private-sentinel'",
            "private-sentinel",
            &"private-sentinel".repeat(5000),
        ] {
            let error = parse_profile(text, &[]).unwrap_err();
            assert!(!error.contains("private-sentinel"));
        }
        let args = [
            "pub",
            "--open-lab",
            "--topic",
            "t",
            "--message",
            "--profile",
        ]
        .map(str::to_owned);
        assert!(crate::cli::parse(args).is_ok());
    }
}
