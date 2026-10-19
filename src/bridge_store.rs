//! Transações locais da bridge: um voo, payload privado e conclusões limitadas.
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{AppResult, bridge_metadata::Envelope, invalid};

const MAX_BYTES: u64 = 2 * 1024 * 1024;

pub fn now_ms() -> AppResult<u64> {
    Ok(u64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Prepared,
    Sent,
    Rel,
    Done,
    Refused,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub envelope: Envelope,
    pub phase: Phase,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub pkid: u16,
    pub fingerprint: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    identity: String,
    saved_ms: u64,
    pub source_known: bool,
    pub destination_known: bool,
    pub discarded_expired: u64,
    pub discarded_path: u64,
    pub pending: Option<Delivery>,
    pub completed: Vec<Completion>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Container {
    version: u8,
    checksum_sha256: String,
    document: Document,
}

struct Disk {
    directory: PathBuf,
    path: PathBuf,
    _lock: File,
}

pub struct Store {
    disk: Arc<Mutex<Disk>>,
    pub document: Document,
}

fn private_file(path: &Path) -> AppResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid("estado da bridge deve ser arquivo real").into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(invalid("estado da bridge exige modo 0600").into());
        }
    }
    Ok(())
}

fn validate(document: &Document) -> AppResult<()> {
    if document.saved_ms > now_ms()? || document.completed.len() > 32 {
        return Err(invalid("relogio regressivo ou limite de conclusoes da bridge").into());
    }
    for (index, item) in document.completed.iter().enumerate() {
        if item.pkid == 0
            || item.fingerprint.len() != 64
            || !item
                .fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || document.completed[..index]
                .iter()
                .any(|old| old.pkid == item.pkid)
        {
            return Err(invalid("conclusao QoS2 invalida").into());
        }
    }
    if let Some(delivery) = &document.pending {
        let item = &delivery.envelope;
        item.metadata.validate()?;
        if !super::valid_topic(&item.source_topic)
            || !super::valid_topic(&item.target_topic)
            || item.payload.len() > super::MAX_PAYLOAD_BYTES
            || item.source_qos > 2
            || (item.source_qos > 0 && item.source_pkid == 0)
            || item.fingerprint.len() != 64
            || !item
                .fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(invalid("encaminhamento persistido invalido").into());
        }
    }
    Ok(())
}

impl Disk {
    fn persist(&self, document: Document) -> AppResult<()> {
        validate(&document)?;
        match fs::symlink_metadata(&self.path) {
            Ok(_) => private_file(&self.path)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let checksum_sha256 = digest(&serde_json::to_vec(&document)?);
        let bytes = serde_json::to_vec(&Container {
            version: 2,
            checksum_sha256,
            document,
        })?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(invalid("journal de encaminhamento acima do limite").into());
        }
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path)?;
        #[cfg(unix)]
        File::open(&self.directory)?.sync_all()?;
        Ok(())
    }
}

impl Store {
    pub async fn open(directory: PathBuf, identity: Vec<u8>) -> AppResult<Self> {
        tokio::task::spawn_blocking(move || {
            super::bridge_multi::private_directory(&directory)?;
            if ["bridge-qos2.json", "bridge.lock"]
                .iter()
                .any(|name| fs::symlink_metadata(directory.join(name)).is_ok())
            {
                return Err(
                    invalid("modo MQTT5 completo exige diretorio sem journal legado").into(),
                );
            }
            let path = directory.join("bridge-forward.json");
            let lock_path = directory.join("bridge-forward.lock");
            let mut options = OpenOptions::new();
            options.create(true).read(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
            }
            let lock = options.open(&lock_path)?;
            private_file(&lock_path)?;
            lock.try_lock_exclusive()?;
            let identity = digest(&identity);
            let document = match fs::symlink_metadata(&path) {
                Ok(_) => {
                    private_file(&path)?;
                    let mut bytes = Vec::new();
                    File::open(&path)?
                        .take(MAX_BYTES + 1)
                        .read_to_end(&mut bytes)?;
                    if bytes.len() as u64 > MAX_BYTES {
                        return Err(invalid("journal acima do limite").into());
                    }
                    let container: Container = serde_json::from_slice(&bytes)?;
                    if container.version != 2
                        || container.document.identity != identity
                        || container.checksum_sha256
                            != digest(&serde_json::to_vec(&container.document)?)
                    {
                        return Err(
                            invalid("journal de outra identidade, versao ou corrompido").into()
                        );
                    }
                    validate(&container.document)?;
                    container.document
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Document {
                    identity,
                    saved_ms: now_ms()?,
                    source_known: false,
                    destination_known: false,
                    discarded_expired: 0,
                    discarded_path: 0,
                    pending: None,
                    completed: Vec::new(),
                },
                Err(error) => return Err(error.into()),
            };
            let disk = Disk {
                directory,
                path,
                _lock: lock,
            };
            disk.persist(document.clone())?;
            Ok(Self {
                disk: Arc::new(Mutex::new(disk)),
                document,
            })
        })
        .await?
    }

    pub fn clock(&self) -> AppResult<u64> {
        let now = now_ms()?;
        if now < self.document.saved_ms {
            return Err(invalid("relogio regressivo; journal preservado").into());
        }
        Ok(now)
    }

    pub async fn commit(&mut self, mut document: Document) -> AppResult<()> {
        document.saved_ms = self.clock()?;
        let disk = Arc::clone(&self.disk);
        let snapshot = document.clone();
        tokio::task::spawn_blocking(move || {
            disk.lock()
                .map_err(|_| invalid("lock interno da bridge indisponivel"))?
                .persist(snapshot)
        })
        .await??;
        self.document = document;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lock_identidade_checksum_e_corrupcao_preservados() {
        let directory = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        }
        let store = Store::open(directory.path().into(), b"rota-a".to_vec())
            .await
            .unwrap();
        assert!(
            Store::open(directory.path().into(), b"rota-a".to_vec())
                .await
                .is_err()
        );
        drop(store);
        assert!(
            Store::open(directory.path().into(), b"rota-b".to_vec())
                .await
                .is_err()
        );
        let path = directory.path().join("bridge-forward.json");
        let original = fs::read(&path).unwrap();
        let mut json: serde_json::Value = serde_json::from_slice(&original).unwrap();
        json["document"]["source_known"] = true.into();
        let corrupted = serde_json::to_vec(&json).unwrap();
        fs::write(&path, &corrupted).unwrap();
        assert!(
            Store::open(directory.path().into(), b"rota-a".to_vec())
                .await
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), corrupted);
        fs::write(&path, original).unwrap();
        assert!(
            Store::open(directory.path().into(), b"rota-a".to_vec())
                .await
                .is_ok()
        );
    }
}
