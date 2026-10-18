//! Private, bounded source-side `QoS` 2 completion journal for mqtt-bridge.
use std::{
    collections::HashSet,
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
const MAX_ENTRIES: usize = 32;
const MAX_JOURNAL_BYTES: u64 = 64 * 1024;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    pkid: u16,
    topic: String,
    payload_sha256: String,
    retain: bool,
    #[serde(default, skip_serializing_if = "Phase::is_await_pubrel")]
    phase: Phase,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    #[default]
    AwaitPubRel,
    PubcompPending,
}

impl Phase {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde skip_serializing_if requires a reference"
    )]
    fn is_await_pubrel(&self) -> bool {
        *self == Self::AwaitPubRel
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u8,
    identity_sha256: String,
    pending: Vec<Pending>,
}

pub struct Journal {
    directory: PathBuf,
    path: PathBuf,
    document: Document,
    _lock: File,
}

impl Journal {
    pub fn open(directory: &Path, identity: &[u8]) -> AppResult<Self> {
        let metadata = fs::symlink_metadata(directory)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid("journal_dir deve ser diretorio real").into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(invalid("journal_dir exige modo 0700 ou mais restrito").into());
            }
        }
        let lock_path = directory.join("bridge.lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let lock = options.open(&lock_path)?;
        check_private_file(&lock_path)?;
        lock.try_lock_exclusive()?;
        let path = directory.join("bridge-qos2.json");
        let identity_sha256 = digest(identity);
        let exists = match fs::symlink_metadata(&path) {
            Ok(_) => true,
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        let document = if exists {
            check_private_file(&path)?;
            let mut bytes = Vec::new();
            File::open(&path)?
                .take(MAX_JOURNAL_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_JOURNAL_BYTES {
                return Err(invalid("journal acima do limite").into());
            }
            let found: Document = serde_json::from_slice(&bytes)?;
            if found.version != 1 || found.identity_sha256 != identity_sha256 {
                return Err(invalid("journal pertence a outra rota ou versao").into());
            }
            validate_document(&found)?;
            found
        } else {
            Document {
                version: 1,
                identity_sha256,
                pending: Vec::new(),
            }
        };
        let journal = Self {
            directory: directory.to_path_buf(),
            path,
            document,
            _lock: lock,
        };
        if !exists {
            journal.persist()?;
        }
        Ok(journal)
    }

    pub fn pending_ids(&self) -> impl Iterator<Item = u16> + '_ {
        self.document.pending.iter().map(|entry| entry.pkid)
    }

    #[cfg(debug_assertions)]
    pub fn pending_topic(&self, pkid: u16) -> Option<&str> {
        self.document
            .pending
            .iter()
            .find(|entry| entry.pkid == pkid)
            .map(|entry| entry.topic.as_str())
    }

    pub fn has_pending(&self) -> bool {
        !self.document.pending.is_empty()
    }

    pub fn already_forwarded(
        &self,
        pkid: u16,
        topic: &str,
        payload: &[u8],
        retain: bool,
    ) -> AppResult<bool> {
        if let Some(entry) = self
            .document
            .pending
            .iter()
            .find(|entry| entry.pkid == pkid)
        {
            if entry.topic != topic
                || entry.payload_sha256 != digest(payload)
                || entry.retain != retain
            {
                return Err(invalid("Packet Identifier pendente com conteudo divergente").into());
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn record(
        &mut self,
        pkid: u16,
        topic: &str,
        payload: &[u8],
        retain: bool,
    ) -> AppResult<()> {
        if pkid == 0 {
            return Err(invalid("Packet Identifier invalido").into());
        }
        if self.already_forwarded(pkid, topic, payload, retain)? {
            return Ok(());
        }
        if self.document.pending.len() >= MAX_ENTRIES {
            return Err(invalid("journal de QoS 2 cheio").into());
        }
        self.document.pending.push(Pending {
            pkid,
            topic: topic.to_owned(),
            payload_sha256: digest(payload),
            retain,
            phase: Phase::AwaitPubRel,
        });
        self.persist()
    }

    pub fn clear_completed_for_new_publish(&mut self, pkid: u16) -> AppResult<()> {
        let before = self.document.pending.len();
        self.document
            .pending
            .retain(|entry| entry.pkid != pkid || entry.phase != Phase::PubcompPending);
        if self.document.pending.len() != before {
            self.persist()?;
        }
        Ok(())
    }

    pub fn make_v5_room(&mut self) -> AppResult<()> {
        if self.document.pending.len() < MAX_ENTRIES {
            return Ok(());
        }
        let position = self
            .document
            .pending
            .iter()
            .position(|entry| entry.phase == Phase::PubcompPending)
            .ok_or_else(|| invalid("journal de QoS 2 cheio"))?;
        self.document.pending.remove(position);
        self.persist()
    }

    pub fn mark_pubcomp_pending(&mut self, pkid: u16) -> AppResult<()> {
        let entry = self
            .document
            .pending
            .iter_mut()
            .find(|entry| entry.pkid == pkid)
            .ok_or_else(|| invalid("PUBREL sem obrigacao no journal"))?;
        if entry.phase == Phase::PubcompPending {
            return Ok(());
        }
        entry.phase = Phase::PubcompPending;
        self.persist()
    }

    pub fn finish(&mut self, pkid: u16) -> AppResult<()> {
        let before = self.document.pending.len();
        self.document.pending.retain(|entry| entry.pkid != pkid);
        if self.document.pending.len() == before {
            return Err(invalid("PUBREL sem obrigacao no journal").into());
        }
        self.persist()
    }

    fn persist(&self) -> AppResult<()> {
        let bytes = serde_json::to_vec(&self.document)?;
        if bytes.len() as u64 > MAX_JOURNAL_BYTES {
            return Err(invalid("journal acima do limite").into());
        }
        let mut temp = tempfile::NamedTempFile::new_in(&self.directory)?;
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        temp.persist(&self.path)?;
        #[cfg(unix)]
        File::open(&self.directory)?.sync_all()?;
        Ok(())
    }
}

fn check_private_file(path: &Path) -> AppResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid("journal ou lock nao e arquivo regular").into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid("journal ou lock exige modo 0600").into());
        }
    }
    Ok(())
}

fn validate_document(document: &Document) -> AppResult<()> {
    if document.pending.len() > MAX_ENTRIES {
        return Err(invalid("journal com entradas acima do limite").into());
    }
    let mut ids = HashSet::new();
    for entry in &document.pending {
        if entry.pkid == 0
            || !ids.insert(entry.pkid)
            || entry.topic.is_empty()
            || entry.topic.len() > 1024
            || entry.payload_sha256.len() != 64
            || !entry
                .payload_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(invalid("entrada invalida no journal").into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Journal;

    #[test]
    fn pending_completion_survives_restart_and_rejects_wrong_identity() {
        let directory = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let mut journal = Journal::open(directory.path(), b"route-a").unwrap();
        journal.record(7, "sensors/ok", b"payload", false).unwrap();
        assert!(
            journal
                .already_forwarded(7, "sensors/ok", b"payload", false)
                .unwrap()
        );
        assert!(
            journal
                .already_forwarded(7, "sensors/ok", b"other", false)
                .is_err()
        );
        assert!(
            journal
                .already_forwarded(7, "sensors/other", b"payload", false)
                .is_err()
        );
        assert!(
            journal
                .already_forwarded(7, "sensors/ok", b"payload", true)
                .is_err()
        );
        assert!(Journal::open(directory.path(), b"route-a").is_err());
        drop(journal);
        assert!(Journal::open(directory.path(), b"route-b").is_err());
        let mut recovered = Journal::open(directory.path(), b"route-a").unwrap();
        assert_eq!(recovered.pending_ids().collect::<Vec<_>>(), vec![7]);
        recovered.finish(7).unwrap();
        assert!(!recovered.has_pending());
    }

    #[test]
    fn corrupt_or_oversized_journal_is_preserved_and_refused() {
        let directory = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let path = directory.path().join("bridge-qos2.json");
        std::fs::write(&path, b"{").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert!(Journal::open(directory.path(), b"route-a").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{");
        std::fs::write(&path, vec![b'x'; 64 * 1024 + 1]).unwrap();
        assert!(Journal::open(directory.path(), b"route-a").is_err());
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 64 * 1024 + 1);
        #[cfg(unix)]
        {
            std::fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(directory.path().join("missing"), &path).unwrap();
            assert!(Journal::open(directory.path(), b"route-a").is_err());
            assert!(
                std::fs::symlink_metadata(&path)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
    }

    #[test]
    fn pending_records_are_bounded_without_overwriting_older_ids() {
        let directory = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let mut journal = Journal::open(directory.path(), b"route-a").unwrap();
        for pkid in 1..=32 {
            journal
                .record(pkid, "sensors/ok", b"payload", false)
                .unwrap();
        }
        journal.record(1, "sensors/ok", b"payload", false).unwrap();
        assert!(journal.record(33, "sensors/ok", b"payload", false).is_err());
        assert_eq!(journal.pending_ids().count(), 32);
        drop(journal);
        let mut recovered = Journal::open(directory.path(), b"route-a").unwrap();
        assert_eq!(recovered.pending_ids().count(), 32);
        recovered.finish(1).unwrap();
        recovered
            .record(33, "sensors/ok", b"payload", false)
            .unwrap();
        assert!(
            recovered
                .already_forwarded(33, "sensors/ok", b"payload", false)
                .unwrap()
        );
        assert!(recovered.finish(1).is_err());
    }

    #[test]
    fn mqtt5_pubcomp_intent_survives_restart_and_id_reuse() {
        let directory = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let mut journal = Journal::open(directory.path(), b"mqtt5-route").unwrap();
        journal.record(7, "sensors/ok", b"first", true).unwrap();
        let path = directory.path().join("bridge-qos2.json");
        assert!(!std::fs::read_to_string(&path).unwrap().contains("phase"));
        journal.mark_pubcomp_pending(7).unwrap();
        drop(journal);

        let mut recovered = Journal::open(directory.path(), b"mqtt5-route").unwrap();
        recovered.mark_pubcomp_pending(7).unwrap();
        assert!(recovered.has_pending());
        recovered.clear_completed_for_new_publish(7).unwrap();
        assert!(!recovered.has_pending());
        recovered.record(7, "sensors/ok", b"second", false).unwrap();
        assert!(
            recovered
                .already_forwarded(7, "sensors/ok", b"second", false)
                .unwrap()
        );
        assert!(recovered.mark_pubcomp_pending(8).is_err());
        drop(recovered);
        assert!(Journal::open(directory.path(), b"mqtt311-route").is_err());
    }
}
