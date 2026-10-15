//! Bounded, opt-in connection profiles. Never accept inline credentials or payloads.
use serde::Deserialize;
use std::{collections::BTreeMap, env, fs::File, io::Read, path::Path};

const ENV_FIELDS: &[(&str, &str)] = &[
    ("host", "MQTT_CLIENT_HOST"),
    ("port", "MQTT_CLIENT_PORT"),
    ("username", "MQTT_CLIENT_USERNAME"),
    ("password_file", "MQTT_CLIENT_PASSWORD_FILE"),
    ("client_id", "MQTT_CLIENT_ID"),
    ("output", "MQTT_CLIENT_OUTPUT"),
    ("qos", "MQTT_CLIENT_QOS"),
    ("keepalive_seconds", "MQTT_CLIENT_KEEPALIVE_SECONDS"),
    ("timeout_seconds", "MQTT_CLIENT_TIMEOUT_SECONDS"),
    ("total_timeout_seconds", "MQTT_CLIENT_TOTAL_TIMEOUT_SECONDS"),
    ("reconnect_attempts", "MQTT_CLIENT_RECONNECT_ATTEMPTS"),
    ("clean_session", "MQTT_CLIENT_CLEAN_SESSION"),
    ("ca", "MQTT_CA_CERT"),
    ("cert", "MQTT_DEVICE_CERT"),
    ("key", "MQTT_DEVICE_KEY"),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    version: u32,
    connection: BTreeMap<String, toml::Value>,
}

pub fn expand(args: Vec<String>) -> Result<Vec<String>, String> {
    expand_with(args, |key| match env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err("variavel de perfil nao e Unicode".into()),
    })
}

fn expand_with(
    args: Vec<String>,
    mut lookup: impl FnMut(&str) -> Result<Option<String>, String>,
) -> Result<Vec<String>, String> {
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
    let environment = environment_overrides(&explicit, &mut lookup)?;
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
        if !explicit.contains(&flag) && !environment.contains_key(key.as_str()) {
            expanded.extend([flag, string]);
        }
    }
    for (key, value) in environment {
        expanded.extend([format!("--{}", key.replace('_', "-")), value]);
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

fn environment_overrides(
    explicit: &std::collections::BTreeSet<String>,
    lookup: &mut impl FnMut(&str) -> Result<Option<String>, String>,
) -> Result<BTreeMap<&'static str, String>, String> {
    let mut environment = BTreeMap::new();
    for &(key, variable) in ENV_FIELDS {
        let flag = format!("--{}", key.replace('_', "-"));
        if !explicit.contains(&flag)
            && let Some(value) = lookup(variable)?
        {
            if value.is_empty() {
                return Err("variavel de perfil vazia".into());
            }
            environment.insert(key, value);
        }
    }
    Ok(environment)
}

fn setting(
    key: &str,
    value: &toml::Value,
    profile_path: &Path,
) -> Result<(String, String), String> {
    let (string, is_path) = match key {
        "host" | "client_id" | "username" | "output" => (value.as_str().map(str::to_owned), false),
        "ca" | "cert" | "key" | "password_file" => (value.as_str().map(str::to_owned), true),
        "port"
        | "qos"
        | "keepalive_seconds"
        | "timeout_seconds"
        | "total_timeout_seconds"
        | "reconnect_attempts" => (value.as_integer().map(|number| number.to_string()), false),
        "clean_session" => (value.as_bool().map(|boolean| boolean.to_string()), false),
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
    use std::collections::BTreeMap;

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

    #[test]
    fn environment_precedence_and_extended_fields() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("connection.toml");
        std::fs::write(&path, "version=1\n[connection]\nhost='file-host'\nport=8883\nusername='device'\nca='ca.crt'\ncert='client.crt'\nkey='client.key'\npassword_file='secret'\nclean_session=false\nreconnect_attempts=2\ntotal_timeout_seconds=90\n").unwrap();
        let mut environment = BTreeMap::from([
            ("MQTT_CLIENT_HOST", "env-host"),
            ("MQTT_CLIENT_PORT", "8884"),
            ("MQTT_CLIENT_PASSWORD_FILE", "relative-env-secret"),
            ("MQTT_CLIENT_RECONNECT_ATTEMPTS", "3"),
            ("MQTT_CA_CERT", "env-ca.crt"),
        ]);
        let args = vec![
            "sub",
            "--topic",
            "sensor/#",
            "--profile",
            path.to_str().unwrap(),
            "--host",
            "cli-host",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let expanded = super::expand_with(args, |key| {
            Ok(environment.get(key).map(ToString::to_string))
        })
        .unwrap();
        let cli = crate::cli::parse(expanded).unwrap();
        assert_eq!(cli.host, "cli-host");
        assert_eq!(cli.port, 8884);
        assert_eq!(cli.ca.unwrap(), std::path::PathBuf::from("env-ca.crt"));
        assert_eq!(
            cli.password_file.unwrap(),
            std::path::PathBuf::from("relative-env-secret")
        );
        assert!(!cli.clean_session);
        assert_eq!(cli.reconnect_attempts, 3);
        assert_eq!(cli.total_timeout_seconds, Some(90));
        environment.insert("MQTT_CLIENT_PORT", "invalid-private-value");
        let args = vec![
            "sub",
            "--topic",
            "sensor/#",
            "--profile",
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let expanded = super::expand_with(args, |key| {
            Ok(environment.get(key).map(ToString::to_string))
        })
        .unwrap();
        let error = crate::cli::parse(expanded).unwrap_err();
        assert!(!error.contains("invalid-private-value"));
    }

    #[test]
    fn environment_errors_do_not_expose_values_or_override_explicit_flags() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("connection.toml");
        std::fs::write(&path, "version=1\n[connection]\nhost='file-host'\nusername='device'\nca='ca.crt'\ncert='client.crt'\nkey='client.key'\n").unwrap();
        let base = vec![
            "sub",
            "--topic",
            "sensor/#",
            "--profile",
            path.to_str().unwrap(),
        ];
        let args = base.iter().map(|s| (*s).to_owned()).collect();
        let error = super::expand_with(args, |key| {
            if key == "MQTT_CLIENT_HOST" {
                Err("variavel de perfil nao e Unicode".into())
            } else {
                Ok(None)
            }
        })
        .unwrap_err();
        assert!(!error.contains("file-host"));
        let args = base
            .into_iter()
            .chain(["--host", "cli-host"])
            .map(str::to_owned)
            .collect();
        let expanded = super::expand_with(args, |key| {
            assert!(
                key != "MQTT_CLIENT_HOST",
                "explicit host must bypass environment"
            );
            Ok(None)
        })
        .unwrap();
        assert_eq!(crate::cli::parse(expanded).unwrap().host, "cli-host");
    }
}
