//! Private, single-operation MQTT 5 `QoS` 1 journal for Linux/WSL.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DOCUMENT_VERSION: u8 = 1;
const MAX_DOCUMENT_BYTES: u64 = 65_536;
const PENDING: &str = "pending.json";
const TEMP: &str = "pending.tmp";
const LOCK: &str = "outbox.lock";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Operation {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub topic: String,
    pub payload: Vec<u8>,
    pub retain: bool,
    pub message_expiry: Option<u32>,
    pub payload_format: Option<u8>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Vec<u8>>,
    pub user_properties: Vec<(String, String)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Stage {
    Prepared,
    MayHaveSent,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PendingQos1 {
    pub operation: Operation,
    pub stage: Stage,
    pub packet_id: u16,
}

#[derive(Serialize, Deserialize)]
struct Document {
    version: u8,
    pending: PendingQos1,
    checksum: [u8; 32],
}

pub(crate) struct Outbox {
    path: PathBuf,
    directory: File,
    _lock: File,
    uid: u32,
}

fn rejected(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}

fn private(metadata: &fs::Metadata, uid: u32, mode: u32, is_dir: bool) -> io::Result<()> {
    if metadata.uid() != uid
        || metadata.permissions().mode() & 0o7777 != mode
        || (is_dir && !metadata.is_dir())
        || (!is_dir && !metadata.is_file())
    {
        return Err(rejected(
            "outbox exige posse do usuario e permissao privada",
        ));
    }
    Ok(())
}

impl Outbox {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        if fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(rejected("outbox nao aceita link simbolico"));
        }
        let uid = crate::credentials::effective_uid()?;
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(path)?;
        private(&directory.metadata()?, uid, 0o700, true)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path.join(LOCK))?;
        private(&lock.metadata()?, uid, 0o600, false)?;
        lock.try_lock_exclusive()?;
        let outbox = Self {
            path: path.to_path_buf(),
            directory,
            _lock: lock,
            uid,
        };
        outbox.remove_stale_temp()?;
        Ok(outbox)
    }

    pub(crate) fn load(&self) -> io::Result<Option<PendingQos1>> {
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(PENDING))
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        private(&file.metadata()?, self.uid, 0o600, false)?;
        let mut bytes = Vec::new();
        file.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "outbox excede limite",
            ));
        }
        let document: Document = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "outbox corrompida"))?;
        let serialized = serde_json::to_vec(&document.pending)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "outbox invalida"))?;
        if document.version != DOCUMENT_VERSION
            || document.pending.packet_id != 1
            || Sha256::digest(serialized).as_slice() != document.checksum
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "outbox corrompida",
            ));
        }
        Ok(Some(document.pending))
    }

    pub(crate) fn prepare(&self, operation: Operation) -> io::Result<PendingQos1> {
        if self.load()?.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "outbox pendente",
            ));
        }
        let pending = PendingQos1 {
            operation,
            stage: Stage::Prepared,
            packet_id: 1,
        };
        self.write(&pending)?;
        Ok(pending)
    }

    pub(crate) fn mark_sending(&self) -> io::Result<PendingQos1> {
        let mut pending = self
            .load()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "outbox ausente"))?;
        if pending.stage == Stage::Prepared {
            pending.stage = Stage::MayHaveSent;
            self.write(&pending)?;
        }
        Ok(pending)
    }

    pub(crate) fn complete(&self) -> io::Result<()> {
        self.load()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "outbox ausente"))?;
        fs::remove_file(self.path.join(PENDING))?;
        self.directory.sync_all()
    }

    fn write(&self, pending: &PendingQos1) -> io::Result<()> {
        let serialized = serde_json::to_vec(pending)?;
        let document = Document {
            version: DOCUMENT_VERSION,
            pending: pending.clone(),
            checksum: Sha256::digest(serialized).into(),
        };
        let bytes = serde_json::to_vec(&document)?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "outbox excede limite",
            ));
        }
        let temp_path = self.path.join(TEMP);
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp_path)?;
        temp.write_all(&bytes)?;
        temp.sync_all()?;
        fs::rename(&temp_path, self.path.join(PENDING))?;
        self.directory.sync_all()
    }

    fn remove_stale_temp(&self) -> io::Result<()> {
        let path = self.path.join(TEMP);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() {
            return Err(rejected("outbox nao aceita link simbolico"));
        }
        private(&metadata, self.uid, 0o600, false)?;
        fs::remove_file(path)?;
        self.directory.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::{Operation, Outbox, Stage};
    use std::{
        fs, io,
        os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    };

    fn operation() -> Operation {
        Operation {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "durable-test".into(),
            topic: "test/outbox".into(),
            payload: vec![0, 0xff, 42],
            retain: false,
            message_expiry: Some(30),
            payload_format: None,
            content_type: Some("application/octet-stream".into()),
            response_topic: None,
            correlation_data: Some(vec![1, 2]),
            user_properties: vec![("trace".into(), "one".into())],
        }
    }

    #[test]
    fn journal_survives_reopen_and_rejects_concurrent_writer() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let metadata = fs::metadata(directory.path()).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o700);
        assert_eq!(metadata.uid(), crate::credentials::effective_uid().unwrap());
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(outbox.load().unwrap().is_none());
        let prepared = outbox.prepare(operation()).unwrap();
        assert!(matches!(prepared.stage, Stage::Prepared));
        assert_eq!(
            outbox.prepare(operation()).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(Outbox::open(directory.path()).is_err());
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        let loaded = outbox.load().unwrap().unwrap();
        assert!(loaded.operation == operation());
        assert!(matches!(loaded.stage, Stage::Prepared));
        assert!(matches!(
            outbox.mark_sending().unwrap().stage,
            Stage::MayHaveSent
        ));
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.load().unwrap().unwrap().stage,
            Stage::MayHaveSent
        ));
        outbox.complete().unwrap();
        assert!(outbox.load().unwrap().is_none());
    }

    #[test]
    fn rejects_symlinks_public_directory_and_corruption() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let link = directory.path().with_extension("link");
        symlink(directory.path(), &link).unwrap();
        assert!(Outbox::open(&link).is_err());
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(Outbox::open(directory.path()).is_err());
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(directory.path()).unwrap();
        symlink("outside", directory.path().join("pending.tmp")).unwrap();
        assert!(outbox.prepare(operation()).is_err());
        assert!(outbox.load().unwrap().is_none());
        fs::remove_file(directory.path().join("pending.tmp")).unwrap();
        outbox.prepare(operation()).unwrap();
        fs::write(directory.path().join("pending.json"), b"{}").unwrap();
        assert!(outbox.load().is_err());
    }
}
