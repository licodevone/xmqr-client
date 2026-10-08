// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
// C27 contract reconstruction: newly authored implementation, not production source.
use crate::args::{Link, Settings};
use std::{
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
    sync::Arc,
};

type SecureResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn validate_secret(secret: &str) -> io::Result<()> {
    if secret.is_empty() || secret.len() > 1024 || secret.chars().any(char::is_control) {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "senha MQTT invalida",
        ))
    } else {
        Ok(())
    }
}

pub fn password(settings: &Settings) -> io::Result<Option<String>> {
    if settings.link == Link::AnonymousLoopback {
        return Ok(None);
    }
    let value = match &settings.password_path {
        Some(path) => read_secret(path)?,
        None => rpassword::prompt_password("Senha MQTT: ")?,
    };
    validate_secret(&value)?;
    Ok(Some(value))
}

fn read_secret(path: &Path) -> io::Result<String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "password-file somente Linux/WSL",
        ))
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        let file = File::open(path)?;
        let meta = file.metadata()?;
        let proc_status = std::fs::read_to_string("/proc/self/status")?;
        let uid = proc_status
            .lines()
            .find_map(|line| line.strip_prefix("Uid:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::PermissionDenied, "UID indisponivel"))?;
        if !meta.is_file() || meta.uid() != uid || meta.mode() & 0o7777 != 0o600 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "arquivo de senha exige regular/current owner/0600",
            ));
        }
        let bytes = bounded(file, 1026)?;
        let mut value = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "senha nao UTF-8"))?;
        if value.ends_with('\n') {
            value.pop();
            if value.ends_with('\r') {
                value.pop();
            }
        }
        validate_secret(&value)?;
        Ok(value)
    }
}

fn bounded(file: File, max: u64) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    file.take(max + 1).read_to_end(&mut output)?;
    if u64::try_from(output.len()).unwrap_or(u64::MAX) > max {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arquivo acima do limite",
        ))
    } else {
        Ok(output)
    }
}

pub fn tls(settings: &Settings) -> SecureResult<rustls::ClientConfig> {
    let read = |path: Option<&std::path::PathBuf>| -> io::Result<Vec<u8>> {
        bounded(
            File::open(
                path.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "PEM ausente"))?,
            )?,
            1024 * 1024,
        )
    };
    let ca = read(settings.ca.as_ref())?;
    let identity = read(settings.cert.as_ref())?;
    let key = read(settings.key.as_ref())?;
    let mut roots = rustls::RootCertStore::empty();
    let authorities = rustls_pemfile::certs(&mut Cursor::new(ca)).collect::<Result<Vec<_>, _>>()?;
    let chain = rustls_pemfile::certs(&mut Cursor::new(identity)).collect::<Result<Vec<_>, _>>()?;
    if authorities.is_empty() || chain.is_empty() {
        return Err("cadeia PEM vazia".into());
    }
    for certificate in authorities {
        roots.add(certificate)?;
    }
    let private = rustls_pemfile::private_key(&mut Cursor::new(key))?.ok_or("chave PEM ausente")?;
    Ok(rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
    .with_root_certificates(roots)
    .with_client_auth_cert(chain, private)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secret_validation_does_not_echo_inputs() {
        for bad in [String::new(), "secret\n".into(), "x".repeat(1025)] {
            let error = validate_secret(&bad).unwrap_err().to_string();
            assert!(!error.contains("secret"));
        }
        assert!(validate_secret(&"é".repeat(512)).is_ok());
    }
    #[test]
    fn pem_bound_is_checked_before_parse_and_missing_inputs_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oversized");
        std::fs::write(&path, vec![0; 1024 * 1024 + 1]).unwrap();
        assert!(bounded(File::open(&path).unwrap(), 1024 * 1024).is_err());
        let settings = crate::args::parse(
            [
                "sub",
                "--topic",
                "t",
                "--username",
                "u",
                "--ca",
                "missing",
                "--cert",
                "missing",
                "--key",
                "missing",
            ]
            .map(str::to_owned),
        )
        .unwrap();
        assert!(tls(&settings).is_err());
    }
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn secret_file_is_rejected_on_this_platform_without_reading() {
        assert_eq!(
            read_secret(Path::new("not-opened")).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn private_file_and_crlf_are_accepted_but_world_readable_is_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("password");
        std::fs::write(&path, b"fixture\r\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(read_secret(&path).unwrap(), "fixture");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_secret(&path).is_err());
    }
}
