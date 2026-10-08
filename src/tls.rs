use std::{
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
    sync::Arc,
};

use rustls::{
    ClientConfig, RootCertStore,
    version::{TLS12, TLS13},
};

use super::cli::Cli;

const MAX_PEM_BYTES: u64 = 1024 * 1024;

/// Trust only the explicitly supplied CA, and always present the device identity.
pub fn build_client_config(
    cli: &Cli,
) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
    let ca_pem = read_limited(cli.ca.as_deref().ok_or("CA ausente no modo seguro")?)?;
    let cert_pem = read_limited(
        cli.cert
            .as_deref()
            .ok_or("certificado ausente no modo seguro")?,
    )?;
    let key_pem = read_limited(
        cli.key
            .as_deref()
            .ok_or("chave privada ausente no modo seguro")?,
    )?;

    let mut roots = RootCertStore::empty();
    let ca_certs =
        rustls_pemfile::certs(&mut Cursor::new(&ca_pem)).collect::<Result<Vec<_>, _>>()?;
    if ca_certs.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "CA sem certificado PEM").into());
    }
    for ca in ca_certs {
        roots.add(ca)?;
    }

    let client_certs =
        rustls_pemfile::certs(&mut Cursor::new(&cert_pem)).collect::<Result<Vec<_>, _>>()?;
    if client_certs.is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "certificado do cliente ausente").into(),
        );
    }
    let private_key = rustls_pemfile::private_key(&mut Cursor::new(&key_pem))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "chave privada PEM ausente"))?;

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&TLS13, &TLS12])?
        .with_root_certificates(roots)
        .with_client_auth_cert(client_certs, private_key)?;
    Ok(config)
}

fn read_limited(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_PEM_BYTES + 1).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_PEM_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("arquivo PEM excede 1 MiB: {}", path.display()),
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::read_limited;
    use std::{fs, io::ErrorKind};

    #[test]
    fn limits_pem_file_before_parsing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.pem");
        fs::write(&path, vec![0u8; 1024 * 1024 + 1]).unwrap();
        assert_eq!(
            read_limited(&path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
    }
}
