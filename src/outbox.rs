//! Private, single-operation MQTT 5 journals for Linux/WSL.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use rumqttc::v5::mqttbytes::v5::{Packet, Publish};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DOCUMENT_VERSION: u8 = 1;
const INBOUND_DOCUMENT_VERSION: u8 = 2;
const MAX_DOCUMENT_BYTES: u64 = 65_536;
const PENDING: &str = "pending.json";
const TEMP: &str = "pending.tmp";
const PENDING_QOS2: &str = "pending-qos2.json";
const TEMP_QOS2: &str = "pending-qos2.tmp";
const SUBSCRIPTION: &str = "subscription.json";
const SUBSCRIPTION_TEMP: &str = "subscription.tmp";
const LOCK: &str = "outbox.lock";
const INBOUND: &str = "subscription-inbound.json";
const INBOUND_TEMP: &str = "subscription-inbound.tmp";
pub(crate) const MAX_INBOUND_RECORDS: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum InboundStage {
    Received,
    ReadyAck,
    AwaitPubRel,
    PubCompPending,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InboundPublication {
    pub packet_id: u16,
    pub qos: u8,
    wire: Vec<u8>,
    identity: [u8; 32],
}

fn invalid_inbound() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "journal inbound invalido")
}

impl InboundPublication {
    pub(crate) fn from_publish(publish: &Publish, topic: &str) -> io::Result<Self> {
        let mut canonical = publish.clone();
        canonical.topic = topic.as_bytes().to_vec().into();
        canonical.dup = false;
        if let Some(properties) = &mut canonical.properties {
            properties.topic_alias = None;
        }
        let mut wire = b"".as_slice().into();
        canonical.write(&mut wire).map_err(|_| invalid_inbound())?;
        let identity = Self::identity(&mut canonical)?;
        let value = Self {
            packet_id: publish.pkid,
            qos: publish.qos as u8,
            wire: wire.to_vec(),
            identity,
        };
        value.publication()?;
        Ok(value)
    }

    fn identity(publish: &mut Publish) -> io::Result<[u8; 32]> {
        Self::identity_version(publish, INBOUND_DOCUMENT_VERSION)
    }

    fn identity_version(publish: &mut Publish, version: u8) -> io::Result<[u8; 32]> {
        // Remaining expiry can decrease on retransmission. Aliases and DUP are
        // connection-local; neither identifies the stored application message.
        if let Some(properties) = &mut publish.properties {
            properties.message_expiry_interval = None;
            if version == INBOUND_DOCUMENT_VERSION {
                // MQTT5 3.3.4/3.8.3.1: IDs can be reordered or changed on replay.
                // Keep their original values in wire; only identity ignores them.
                properties.subscription_identifiers.clear();
            }
        }
        let mut wire = b"".as_slice().into();
        publish.write(&mut wire).map_err(|_| invalid_inbound())?;
        Ok(Sha256::digest(&wire).into())
    }

    pub(crate) fn publication(&self) -> io::Result<Publish> {
        self.publication_version(INBOUND_DOCUMENT_VERSION)
    }

    fn publication_version(&self, version: u8) -> io::Result<Publish> {
        if self.wire.len() > 65536 || self.packet_id == 0 || !(1..=2).contains(&self.qos) {
            return Err(invalid_inbound());
        }
        let mut wire = self.wire.as_slice().into();
        let Packet::Publish(publish) =
            Packet::read(&mut wire, Some(65536)).map_err(|_| invalid_inbound())?
        else {
            return Err(invalid_inbound());
        };
        let topic = std::str::from_utf8(&publish.topic).map_err(|_| invalid_inbound())?;
        if !wire.is_empty()
            || publish.pkid != self.packet_id
            || publish.qos as u8 != self.qos
            || publish.dup
            || topic.is_empty()
            || topic.len() > 1024
            || topic.contains(['+', '#', '\0'])
            || publish.payload.len() > 4096
            || publish
                .properties
                .as_ref()
                .is_some_and(|p| p.topic_alias.is_some())
            || Self::identity_version(&mut publish.clone(), version)? != self.identity
        {
            return Err(invalid_inbound());
        }
        Ok(publish)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PendingInbound {
    pub publication: InboundPublication,
    pub stage: InboundStage,
}

#[derive(Serialize, Deserialize)]
struct InboundDocument {
    version: u8,
    subscription_checksum: [u8; 32],
    entries: Vec<PendingInbound>,
    checksum: [u8; 32],
}

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

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Qos2Stage {
    Prepared,
    PublishMayHaveSent,
    PubRelReady,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PendingQos2 {
    pub operation: Operation,
    pub stage: Qos2Stage,
    pub packet_id: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SubscriptionSpec {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub session_expiry: u32,
    pub filters: Vec<String>,
    pub qos: u8,
    pub retain_handling: u8,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub identifier: Option<usize>,
    pub user_properties: Vec<(String, String)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SubscriptionStage {
    Prepared,
    Active,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PendingSubscription {
    pub spec: SubscriptionSpec,
    pub stage: SubscriptionStage,
}

#[derive(Serialize, Deserialize)]
struct SubscriptionDocument {
    version: u8,
    pending: PendingSubscription,
    checksum: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct Document {
    version: u8,
    pending: PendingQos1,
    checksum: [u8; 32],
}

#[derive(Serialize, Deserialize)]
struct Qos2Document {
    version: u8,
    pending: PendingQos2,
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
        outbox.reject_orphaned_inbound()?;
        outbox.remove_stale_temp(TEMP)?;
        outbox.remove_stale_temp(TEMP_QOS2)?;
        outbox.remove_stale_temp(SUBSCRIPTION_TEMP)?;
        outbox.remove_stale_temp(INBOUND_TEMP)?;
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
        if self.load()?.is_some()
            || self.load_qos2()?.is_some()
            || self.load_subscription()?.is_some()
        {
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

    pub(crate) fn load_qos2(&self) -> io::Result<Option<PendingQos2>> {
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(PENDING_QOS2))
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
        let document: Qos2Document = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "outbox QoS 2 corrompida"))?;
        let serialized = serde_json::to_vec(&document.pending)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "outbox QoS 2 invalida"))?;
        if document.version != DOCUMENT_VERSION
            || document.pending.packet_id != 1
            || Sha256::digest(serialized).as_slice() != document.checksum
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "outbox QoS 2 corrompida",
            ));
        }
        Ok(Some(document.pending))
    }

    pub(crate) fn prepare_qos2(&self, operation: Operation) -> io::Result<PendingQos2> {
        if self.load()?.is_some()
            || self.load_qos2()?.is_some()
            || self.load_subscription()?.is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "outbox pendente",
            ));
        }
        let pending = PendingQos2 {
            operation,
            stage: Qos2Stage::Prepared,
            packet_id: 1,
        };
        self.write_qos2(&pending)?;
        Ok(pending)
    }

    pub(crate) fn mark_qos2_sending(&self) -> io::Result<PendingQos2> {
        let mut pending = self
            .load_qos2()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "outbox QoS 2 ausente"))?;
        if pending.stage == Qos2Stage::Prepared {
            pending.stage = Qos2Stage::PublishMayHaveSent;
            self.write_qos2(&pending)?;
        }
        Ok(pending)
    }

    pub(crate) fn mark_pubrel_ready(&self) -> io::Result<PendingQos2> {
        let mut pending = self
            .load_qos2()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "outbox QoS 2 ausente"))?;
        if pending.stage != Qos2Stage::PublishMayHaveSent {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "fase QoS 2 invalida",
            ));
        }
        pending.stage = Qos2Stage::PubRelReady;
        self.write_qos2(&pending)?;
        Ok(pending)
    }

    pub(crate) fn complete_qos2(&self) -> io::Result<()> {
        self.load_qos2()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "outbox QoS 2 ausente"))?;
        fs::remove_file(self.path.join(PENDING_QOS2))?;
        self.directory.sync_all()
    }

    pub(crate) fn load_subscription(&self) -> io::Result<Option<PendingSubscription>> {
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(SUBSCRIPTION))
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
                "manifesto excede limite",
            ));
        }
        let document: SubscriptionDocument = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "manifesto corrompido"))?;
        let serialized = serde_json::to_vec(&document.pending)?;
        if document.version != DOCUMENT_VERSION
            || Sha256::digest(serialized).as_slice() != document.checksum
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "manifesto corrompido",
            ));
        }
        Ok(Some(document.pending))
    }

    pub(crate) fn prepare_subscription(
        &self,
        spec: SubscriptionSpec,
    ) -> io::Result<PendingSubscription> {
        self.reject_orphaned_inbound()?;
        if self.load()?.is_some()
            || self.load_qos2()?.is_some()
            || self.load_subscription()?.is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "journal pendente",
            ));
        }
        let pending = PendingSubscription {
            spec,
            stage: SubscriptionStage::Prepared,
        };
        self.write_subscription(&pending)?;
        Ok(pending)
    }

    pub(crate) fn activate_subscription(&self) -> io::Result<()> {
        let mut pending = self.load_subscription()?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "manifesto de assinatura ausente")
        })?;
        pending.stage = SubscriptionStage::Active;
        self.write_subscription(&pending)
    }

    pub(crate) fn remove_unactivated_subscription(&self) -> io::Result<()> {
        let pending = self.load_subscription()?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "manifesto de assinatura ausente")
        })?;
        if pending.stage != SubscriptionStage::Prepared {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "assinatura ja ativa",
            ));
        }
        fs::remove_file(self.path.join(SUBSCRIPTION))?;
        self.directory.sync_all()
    }

    fn write_subscription(&self, pending: &PendingSubscription) -> io::Result<()> {
        let serialized = serde_json::to_vec(pending)?;
        let document = SubscriptionDocument {
            version: DOCUMENT_VERSION,
            pending: pending.clone(),
            checksum: Sha256::digest(serialized).into(),
        };
        let bytes = serde_json::to_vec(&document)?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "manifesto excede limite",
            ));
        }
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(SUBSCRIPTION_TEMP))?;
        temp.write_all(&bytes)?;
        temp.sync_all()?;
        fs::rename(
            self.path.join(SUBSCRIPTION_TEMP),
            self.path.join(SUBSCRIPTION),
        )?;
        self.directory.sync_all()
    }

    fn reject_orphaned_inbound(&self) -> io::Result<()> {
        match fs::symlink_metadata(self.path.join(SUBSCRIPTION)) {
            Ok(_) => return Ok(()), // existing manifest validation keeps its own errors
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
        }
        match fs::symlink_metadata(self.path.join(INBOUND)) {
            Ok(_) => Err(invalid_inbound()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn inbound_subscription_checksum(&self) -> io::Result<[u8; 32]> {
        let subscription = self.load_subscription()?.ok_or_else(invalid_inbound)?;
        if subscription.stage != SubscriptionStage::Active {
            return Err(invalid_inbound());
        }
        Ok(Sha256::digest(serde_json::to_vec(&subscription.spec)?).into())
    }

    pub(crate) fn load_inbound(&self) -> io::Result<Vec<PendingInbound>> {
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(INBOUND))
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        private(&file.metadata()?, self.uid, 0o600, false)?;
        let mut bytes = Vec::new();
        file.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(invalid_inbound());
        }
        let mut document: InboundDocument =
            serde_json::from_slice(&bytes).map_err(|_| invalid_inbound())?;
        let serialized = serde_json::to_vec(&(document.subscription_checksum, &document.entries))?;
        if !matches!(document.version, 1 | INBOUND_DOCUMENT_VERSION)
            || document.subscription_checksum != self.inbound_subscription_checksum()?
            || Sha256::digest(serialized).as_slice() != document.checksum
        {
            return Err(invalid_inbound());
        }
        if document.version == 1 {
            // Validate the frozen draft algorithm first, then normalize in RAM.
            // Loading never rewrites original metadata or bytes. A subsequent
            // existing durable transition writes a version2 document atomically.
            for entry in &mut document.entries {
                let mut publication = entry.publication.publication_version(1)?;
                entry.publication.identity = InboundPublication::identity(&mut publication)?;
            }
        }
        Self::validate_inbound(&document.entries)?;
        Ok(document.entries)
    }

    fn validate_inbound(entries: &[PendingInbound]) -> io::Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        if entries.len() > MAX_INBOUND_RECORDS {
            return Err(invalid_inbound());
        }
        for entry in entries {
            entry.publication.publication()?;
            if !ids.insert(entry.publication.packet_id)
                || (entry.stage == InboundStage::ReadyAck && entry.publication.qos != 1)
                || (matches!(
                    entry.stage,
                    InboundStage::AwaitPubRel | InboundStage::PubCompPending
                ) && entry.publication.qos != 2)
            {
                return Err(invalid_inbound());
            }
        }
        Ok(())
    }

    fn write_inbound(&self, entries: Vec<PendingInbound>) -> io::Result<()> {
        Self::validate_inbound(&entries)?;
        let subscription_checksum = self.inbound_subscription_checksum()?;
        let checksum =
            Sha256::digest(serde_json::to_vec(&(subscription_checksum, &entries))?).into();
        let bytes = serde_json::to_vec(&InboundDocument {
            version: INBOUND_DOCUMENT_VERSION,
            subscription_checksum,
            entries,
            checksum,
        })?;
        if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "journal inbound excede limite",
            ));
        }
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(self.path.join(INBOUND_TEMP))?;
        temp.write_all(&bytes)?;
        temp.sync_all()?;
        fs::rename(self.path.join(INBOUND_TEMP), self.path.join(INBOUND))?;
        self.directory.sync_all()
    }

    pub(crate) fn receive_inbound(
        &self,
        publication: InboundPublication,
    ) -> io::Result<PendingInbound> {
        publication.publication()?;
        let mut entries = self.load_inbound()?;
        // These durable phases have already discharged application delivery.
        // A lost terminal ACK is safely answered on replay (QoS1 may duplicate;
        // an unknown PUBREL gets 0x92), so crash-stale completion markers must
        // not exhaust the bounded journal forever. Never discard live phases.
        entries.retain(|entry| {
            !matches!(
                entry.stage,
                InboundStage::ReadyAck | InboundStage::PubCompPending
            )
        });
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.publication.packet_id == publication.packet_id)
        {
            let prior = &entries[index];
            if prior.stage == InboundStage::Received || prior.publication.qos == 2 {
                if prior.publication.identity != publication.identity {
                    return Err(invalid_inbound());
                }
                return Ok(prior.clone());
            }
            // QoS1 permits duplicates; after terminal QoS2 completion a PUBLISH
            // with a reused identifier is a new application message, even DUP.
            entries.remove(index);
        }
        if entries.len() >= MAX_INBOUND_RECORDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "journal inbound excede limite",
            ));
        }
        let pending = PendingInbound {
            publication,
            stage: InboundStage::Received,
        };
        entries.push(pending.clone());
        self.write_inbound(entries)?;
        Ok(pending)
    }

    pub(crate) fn inbound_output_ready(&self, packet_id: u16) -> io::Result<()> {
        let mut entries = self.load_inbound()?;
        let entry = entries
            .iter_mut()
            .find(|entry| entry.publication.packet_id == packet_id)
            .ok_or_else(invalid_inbound)?;
        if entry.stage != InboundStage::Received {
            return Err(invalid_inbound());
        }
        entry.stage = if entry.publication.qos == 1 {
            InboundStage::ReadyAck
        } else {
            InboundStage::AwaitPubRel
        };
        self.write_inbound(entries)
    }

    pub(crate) fn inbound_pubrel(&self, packet_id: u16) -> io::Result<bool> {
        let mut entries = self.load_inbound()?;
        let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.publication.packet_id == packet_id)
        else {
            return Ok(false);
        };
        if entry.publication.qos != 2
            || !matches!(
                entry.stage,
                InboundStage::AwaitPubRel | InboundStage::PubCompPending
            )
        {
            return Err(invalid_inbound());
        }
        entry.stage = InboundStage::PubCompPending;
        self.write_inbound(entries)?;
        Ok(true)
    }

    pub(crate) fn inbound_ack_written(&self, packet_id: u16, qos: u8) -> io::Result<()> {
        let mut entries = self.load_inbound()?;
        let Some(index) = entries
            .iter()
            .position(|entry| entry.publication.packet_id == packet_id)
        else {
            return Ok(()); // unknown PUBREL answered with 0x92
        };
        let entry = &entries[index];
        let expected = if qos == 1 {
            InboundStage::ReadyAck
        } else {
            InboundStage::PubCompPending
        };
        if entry.publication.qos != qos || entry.stage != expected {
            return Err(invalid_inbound());
        }
        entries.remove(index);
        self.write_inbound(entries)
    }

    fn write_qos2(&self, pending: &PendingQos2) -> io::Result<()> {
        let serialized = serde_json::to_vec(pending)?;
        let document = Qos2Document {
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
        let temp_path = self.path.join(TEMP_QOS2);
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp_path)?;
        temp.write_all(&bytes)?;
        temp.sync_all()?;
        fs::rename(&temp_path, self.path.join(PENDING_QOS2))?;
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

    fn remove_stale_temp(&self, name: &str) -> io::Result<()> {
        let path = self.path.join(name);
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
    use super::{INBOUND, INBOUND_TEMP, InboundPublication, InboundStage, MAX_DOCUMENT_BYTES};
    use super::{Operation, Outbox, Qos2Stage, Stage, SubscriptionSpec, SubscriptionStage};
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

    fn subscription() -> SubscriptionSpec {
        SubscriptionSpec {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "durable-sub".into(),
            username: None,
            session_expiry: 60,
            filters: vec!["test/#".into()],
            qos: 1,
            retain_handling: 0,
            no_local: false,
            retain_as_published: false,
            identifier: None,
            user_properties: Vec::new(),
        }
    }

    fn receiver() -> (tempfile::TempDir, Outbox) {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(directory.path()).unwrap();
        outbox.prepare_subscription(subscription()).unwrap();
        outbox.activate_subscription().unwrap();
        (directory, outbox)
    }

    fn incoming(id: u16, qos: u8, payload: Vec<u8>) -> InboundPublication {
        use rumqttc::v5::mqttbytes::{QoS, v5::Publish};
        let mut publish = Publish::new(
            "inbound/test",
            if qos == 1 {
                QoS::AtLeastOnce
            } else {
                QoS::ExactlyOnce
            },
            payload,
            None,
        );
        publish.pkid = id;
        InboundPublication::from_publish(&publish, "inbound/test").unwrap()
    }

    #[test]
    fn inbound_review_subscription_property_cursor_preserves_three_ids_and_binary_payload() {
        assert_subscription_property_roundtrip(&[1, 1, 2]);
    }

    #[test]
    fn inbound_review_subscription_property_cursor_preserves_all_vbi_widths() {
        assert_subscription_property_roundtrip(&[1, 128, 16_384, 2_097_152]);
    }

    fn assert_subscription_property_roundtrip(identifiers: &[usize]) {
        use rumqttc::v5::mqttbytes::{
            QoS,
            v5::{Packet, Publish, PublishProperties},
        };
        let mut publish = Publish::new(
            "inbound/test",
            QoS::ExactlyOnce,
            vec![0, 255, 3, 0],
            Some(PublishProperties {
                subscription_identifiers: identifiers.to_vec(),
                content_type: Some(String::new()),
                ..PublishProperties::default()
            }),
        );
        publish.pkid = 7;
        let mut wire = b"".as_slice().into();
        publish.write(&mut wire).unwrap();
        let Packet::Publish(decoded) = Packet::read(&mut wire, Some(65536)).unwrap() else {
            panic!("expected PUBLISH");
        };
        assert_eq!(
            decoded
                .properties
                .as_ref()
                .unwrap()
                .subscription_identifiers,
            identifiers
        );
        assert_eq!(
            decoded.properties.as_ref().unwrap().content_type.as_deref(),
            Some("")
        );
        assert_eq!(decoded.payload.as_ref(), &[0, 255, 3, 0]);
        assert!(wire.is_empty());
    }

    #[test]
    fn inbound_review_orphan_refuses_open_without_touching_temporary_or_document() {
        let (directory, outbox) = receiver();
        outbox.receive_inbound(incoming(7, 2, vec![1])).unwrap();
        let path = directory.path().join(INBOUND);
        let before = fs::read(&path).unwrap();
        let temporary = directory.path().join(INBOUND_TEMP);
        fs::write(&temporary, b"owned-orphan-temporary").unwrap();
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)).unwrap();
        fs::remove_file(directory.path().join(super::SUBSCRIPTION)).unwrap();
        drop(outbox);
        assert!(Outbox::open(directory.path()).is_err());
        assert!(!directory.path().join(super::SUBSCRIPTION).exists());
        assert_eq!(fs::read(path).unwrap(), before);
        assert_eq!(fs::read(temporary).unwrap(), b"owned-orphan-temporary");
    }

    #[test]
    fn inbound_review_orphan_refuses_new_manifest_preparation() {
        let (directory, outbox) = receiver();
        outbox.receive_inbound(incoming(7, 2, vec![1])).unwrap();
        let before = fs::read(directory.path().join(INBOUND)).unwrap();
        fs::remove_file(directory.path().join(super::SUBSCRIPTION)).unwrap();
        assert!(outbox.prepare_subscription(subscription()).is_err());
        assert!(!directory.path().join(super::SUBSCRIPTION).exists());
        assert_eq!(fs::read(directory.path().join(INBOUND)).unwrap(), before);
    }

    #[test]
    fn inbound_review_mutable_subscription_identifiers_do_not_change_obligation() {
        use rumqttc::v5::mqttbytes::{
            QoS,
            v5::{Publish, PublishProperties},
        };
        let (directory, outbox) = receiver();
        let mut publish = Publish::new(
            "inbound/test",
            QoS::ExactlyOnce,
            vec![1, 2],
            Some(PublishProperties {
                subscription_identifiers: vec![1, 2],
                user_properties: vec![
                    ("key".into(), "first".into()),
                    ("key".into(), "second".into()),
                ],
                ..PublishProperties::default()
            }),
        );
        publish.pkid = 7;
        outbox
            .receive_inbound(InboundPublication::from_publish(&publish, "inbound/test").unwrap())
            .unwrap();
        outbox.inbound_output_ready(7).unwrap();
        let before = fs::read(directory.path().join(INBOUND)).unwrap();
        for identifiers in [vec![2, 1], vec![1], vec![]] {
            publish
                .properties
                .as_mut()
                .unwrap()
                .subscription_identifiers = identifiers;
            let replay = InboundPublication::from_publish(&publish, "inbound/test").unwrap();
            assert!(matches!(
                outbox.receive_inbound(replay).unwrap().stage,
                InboundStage::AwaitPubRel
            ));
            assert_eq!(fs::read(directory.path().join(INBOUND)).unwrap(), before);
            let original = outbox.load_inbound().unwrap()[0]
                .publication
                .publication()
                .unwrap();
            assert_eq!(
                original.properties.unwrap().subscription_identifiers,
                vec![1, 2]
            );
        }
        publish.payload = vec![9].into();
        assert!(
            outbox
                .receive_inbound(
                    InboundPublication::from_publish(&publish, "inbound/test").unwrap()
                )
                .is_err()
        );
    }

    #[test]
    fn inbound_review_v1_draft_is_read_only_until_explicit_v2_transition() {
        use sha2::{Digest, Sha256};
        let (directory, outbox) = receiver();
        let mut publication = incoming(7, 2, vec![1, 2]);
        let mut original = publication.publication().unwrap();
        original.properties = Some(rumqttc::v5::mqttbytes::v5::PublishProperties {
            subscription_identifiers: vec![1, 2],
            ..Default::default()
        });
        publication = InboundPublication::from_publish(&original, "inbound/test").unwrap();
        // Frozen v1 algorithm: canonical wire hash removes expiry only, not IDs.
        let mut legacy = original.clone();
        legacy.dup = false;
        let mut wire = b"".as_slice().into();
        legacy.write(&mut wire).unwrap();
        publication.identity = Sha256::digest(&wire).into();
        let entries = vec![super::PendingInbound {
            publication,
            stage: InboundStage::AwaitPubRel,
        }];
        let subscription_checksum = outbox.inbound_subscription_checksum().unwrap();
        let checksum =
            Sha256::digest(serde_json::to_vec(&(subscription_checksum, &entries)).unwrap()).into();
        let bytes = serde_json::to_vec(&super::InboundDocument {
            version: 1,
            subscription_checksum,
            entries,
            checksum,
        })
        .unwrap();
        let path = directory.path().join(INBOUND);
        fs::write(&path, &bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let mut unknown: super::InboundDocument = serde_json::from_slice(&bytes).unwrap();
        unknown.version = 99;
        fs::write(&path, serde_json::to_vec(&unknown).unwrap()).unwrap();
        assert!(outbox.load_inbound().is_err());
        let mut corrupt: super::InboundDocument = serde_json::from_slice(&bytes).unwrap();
        corrupt.entries[0].publication.identity = [0; 32];
        corrupt.checksum = Sha256::digest(
            serde_json::to_vec(&(corrupt.subscription_checksum, &corrupt.entries)).unwrap(),
        )
        .into();
        fs::write(&path, serde_json::to_vec(&corrupt).unwrap()).unwrap();
        assert!(outbox.load_inbound().is_err());
        fs::write(&path, &bytes).unwrap();
        let loaded = outbox.load_inbound().unwrap();
        assert_eq!(
            loaded[0]
                .publication
                .publication()
                .unwrap()
                .properties
                .unwrap()
                .subscription_identifiers,
            vec![1, 2]
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(outbox.inbound_pubrel(7).unwrap());
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(document["version"], 2);
    }

    #[test]
    fn inbound_qos2_recovers_both_durable_ack_boundaries_and_new_id_reuse() {
        let (directory, outbox) = receiver();
        let first = incoming(7, 2, vec![0, 255, 10]);
        assert!(outbox.receive_inbound(first.clone()).is_ok());
        assert!(outbox.inbound_ack_written(7, 2).is_err());
        assert!(matches!(
            outbox.load_inbound().unwrap()[0].stage,
            InboundStage::Received
        ));
        outbox.inbound_output_ready(7).unwrap();
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.receive_inbound(first).unwrap().stage,
            InboundStage::AwaitPubRel
        ));
        assert!(outbox.inbound_pubrel(7).unwrap());
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(outbox.inbound_pubrel(7).unwrap());
        outbox.inbound_ack_written(7, 2).unwrap();
        assert!(outbox.load_inbound().unwrap().is_empty());
        assert!(!outbox.inbound_pubrel(7).unwrap());
        assert!(matches!(
            outbox
                .receive_inbound(incoming(7, 2, vec![8]))
                .unwrap()
                .stage,
            InboundStage::Received
        ));
    }

    #[test]
    fn inbound_qos1_requires_durable_output_transition_before_cleanup() {
        let (_directory, outbox) = receiver();
        outbox.receive_inbound(incoming(1, 1, vec![1])).unwrap();
        assert!(outbox.inbound_ack_written(1, 1).is_err());
        outbox.inbound_output_ready(1).unwrap();
        outbox.inbound_ack_written(1, 1).unwrap();
        assert!(outbox.load_inbound().unwrap().is_empty());
    }

    #[test]
    fn inbound_qos1_undelivered_record_cannot_be_overwritten_by_conflicting_identifier() {
        let (directory, outbox) = receiver();
        outbox.receive_inbound(incoming(7, 1, vec![1])).unwrap();
        let before = fs::read(directory.path().join(INBOUND)).unwrap();
        assert!(outbox.receive_inbound(incoming(7, 1, vec![2])).is_err());
        assert_eq!(fs::read(directory.path().join(INBOUND)).unwrap(), before);
        assert!(matches!(
            outbox
                .receive_inbound(incoming(7, 1, vec![1]))
                .unwrap()
                .stage,
            InboundStage::Received
        ));
    }

    #[test]
    fn inbound_terminal_crash_markers_do_not_exhaust_live_record_quota() {
        let (_directory, outbox) = receiver();
        outbox.receive_inbound(incoming(1, 2, vec![1])).unwrap();
        outbox.inbound_output_ready(1).unwrap();
        outbox.receive_inbound(incoming(2, 2, vec![2])).unwrap();
        for id in 3..=100 {
            outbox.receive_inbound(incoming(id, 2, vec![3])).unwrap();
            outbox.inbound_output_ready(id).unwrap();
            outbox.inbound_pubrel(id).unwrap();
            // Simulate death after durable PUBREL but before cleanup. A new
            // publication permits collection, not redelivery of old output.
        }
        outbox.receive_inbound(incoming(101, 1, vec![4])).unwrap();
        outbox.inbound_output_ready(101).unwrap();
        outbox.receive_inbound(incoming(102, 1, vec![5])).unwrap();
        let entries = outbox.load_inbound().unwrap();
        assert_eq!(entries.len(), 3);
        assert!(
            entries.iter().any(|entry| entry.publication.packet_id == 1
                && entry.stage == InboundStage::AwaitPubRel)
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.publication.packet_id == 2
                    && entry.stage == InboundStage::Received)
        );
        assert!(!outbox.inbound_pubrel(100).unwrap());
        assert!(outbox.receive_inbound(incoming(1, 2, vec![9])).is_err());
    }

    #[test]
    fn inbound_canonical_wire_preserves_binary_metadata_and_expiry_replay_identity() {
        use rumqttc::v5::mqttbytes::{
            QoS,
            v5::{Publish, PublishProperties},
        };
        let (_directory, outbox) = receiver();
        let mut publish = Publish::new(
            "",
            QoS::ExactlyOnce,
            vec![0, 255, 10],
            Some(PublishProperties {
                message_expiry_interval: Some(30),
                content_type: Some("application/octet-stream".into()),
                response_topic: Some("reply/x".into()),
                correlation_data: Some(vec![0, 255, 0].into()),
                user_properties: vec![
                    ("same".into(), "first".into()),
                    ("same".into(), "second".into()),
                ],
                subscription_identifiers: vec![1, 268_435_455],
                topic_alias: Some(7),
                ..PublishProperties::default()
            }),
        );
        publish.pkid = 7;
        publish.retain = true;
        publish.dup = true;
        let canonical = InboundPublication::from_publish(&publish, "inbound/test").unwrap();
        let restored = canonical.publication().unwrap();
        assert_eq!(restored.payload.as_ref(), &[0, 255, 10]);
        assert_eq!(restored.topic.as_ref(), b"inbound/test");
        assert!(!restored.dup);
        let properties = restored.properties.unwrap();
        assert_eq!(properties.message_expiry_interval, Some(30));
        assert_eq!(
            properties.correlation_data.as_deref(),
            Some(&[0, 255, 0][..])
        );
        assert_eq!(properties.subscription_identifiers, [1, 268_435_455]);
        assert_eq!(properties.user_properties[1].1, "second");
        assert_eq!(properties.topic_alias, None);
        outbox.receive_inbound(canonical).unwrap();
        outbox.inbound_output_ready(7).unwrap();
        publish.properties.as_mut().unwrap().message_expiry_interval = Some(20);
        let replay = InboundPublication::from_publish(&publish, "inbound/test").unwrap();
        assert!(matches!(
            outbox.receive_inbound(replay).unwrap().stage,
            InboundStage::AwaitPubRel
        ));
        publish
            .properties
            .as_mut()
            .unwrap()
            .user_properties
            .swap(0, 1);
        let changed = InboundPublication::from_publish(&publish, "inbound/test").unwrap();
        assert!(outbox.receive_inbound(changed).is_err());
    }

    #[test]
    fn inbound_record_and_byte_quotas_refuse_without_replacing_committed_data() {
        let (directory, outbox) = receiver();
        for id in 1..=32 {
            outbox.receive_inbound(incoming(id, 2, vec![1])).unwrap();
        }
        let path = directory.path().join(INBOUND);
        let before = fs::read(&path).unwrap();
        assert!(outbox.receive_inbound(incoming(33, 2, vec![1])).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        let (directory, outbox) = receiver();
        let path = directory.path().join(INBOUND);
        let mut refused = false;
        for id in 1..=32 {
            let before = fs::read(&path).ok();
            if outbox
                .receive_inbound(incoming(id, 2, vec![255; 4096]))
                .is_err()
            {
                assert_eq!(fs::read(&path).ok(), before);
                refused = true;
                break;
            }
        }
        assert!(refused);
        assert!(fs::metadata(&path).unwrap().len() <= MAX_DOCUMENT_BYTES);
    }

    #[test]
    fn inbound_identity_corruption_permissions_symlink_and_lock_are_rejected() {
        let (directory, outbox) = receiver();
        outbox.receive_inbound(incoming(7, 2, vec![1])).unwrap();
        assert!(Outbox::open(directory.path()).is_err());
        let path = directory.path().join(INBOUND);
        let original = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(outbox.load_inbound().is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{}").unwrap();
        assert!(outbox.load_inbound().is_err());
        fs::write(&path, &original).unwrap();
        let mut pending = outbox.load_subscription().unwrap().unwrap();
        pending.spec.client_id.push('x');
        outbox.write_subscription(&pending).unwrap();
        assert!(outbox.load_inbound().is_err());
        fs::remove_file(&path).unwrap();
        symlink("missing-owned-fixture", &path).unwrap();
        assert!(outbox.load_inbound().is_err());
    }

    #[test]
    fn inbound_failed_commit_preserves_received_phase_and_old_document() {
        let (directory, outbox) = receiver();
        outbox.receive_inbound(incoming(7, 2, vec![1])).unwrap();
        let before = fs::read(directory.path().join(INBOUND)).unwrap();
        fs::create_dir(directory.path().join(INBOUND_TEMP)).unwrap();
        assert!(outbox.inbound_output_ready(7).is_err());
        assert_eq!(fs::read(directory.path().join(INBOUND)).unwrap(), before);
        assert!(matches!(
            outbox.load_inbound().unwrap()[0].stage,
            InboundStage::Received
        ));
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

    #[test]
    fn qos2_journal_survives_both_stages_and_excludes_qos1() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.prepare_qos2(operation()).unwrap().stage,
            Qos2Stage::Prepared
        ));
        assert_eq!(
            outbox.prepare(operation()).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.mark_qos2_sending().unwrap().stage,
            Qos2Stage::PublishMayHaveSent
        ));
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.mark_pubrel_ready().unwrap().stage,
            Qos2Stage::PubRelReady
        ));
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(outbox.mark_pubrel_ready().is_err());
        assert!(matches!(
            outbox.load_qos2().unwrap().unwrap().stage,
            Qos2Stage::PubRelReady
        ));
        outbox.complete_qos2().unwrap();
        assert!(outbox.load_qos2().unwrap().is_none());
        outbox.prepare(operation()).unwrap();
        assert_eq!(
            outbox.prepare_qos2(operation()).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
    }

    #[test]
    fn qos2_rejects_corruption_and_symlink_without_losing_qos1() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(directory.path()).unwrap();
        let pending = directory.path().join("pending-qos2.json");
        symlink("outside", &pending).unwrap();
        assert!(outbox.load_qos2().is_err());
        fs::remove_file(&pending).unwrap();
        let qos1 = outbox.prepare(operation()).unwrap();
        assert_eq!(qos1.packet_id, 1);
        assert_eq!(
            outbox.prepare_qos2(operation()).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        outbox.complete().unwrap();
        outbox.prepare_qos2(operation()).unwrap();
        fs::write(&pending, b"{}").unwrap();
        assert_eq!(
            outbox.load_qos2().err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn subscription_manifest_survives_reopen_and_rejects_corruption() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(directory.path()).unwrap();
        let prepared = outbox.prepare_subscription(subscription()).unwrap();
        assert!(matches!(prepared.stage, SubscriptionStage::Prepared));
        assert!(Outbox::open(directory.path()).is_err());
        assert!(outbox.prepare(operation()).is_err());
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert_eq!(
            outbox.load_subscription().unwrap().unwrap().spec,
            subscription()
        );
        outbox.activate_subscription().unwrap();
        drop(outbox);
        let outbox = Outbox::open(directory.path()).unwrap();
        assert!(matches!(
            outbox.load_subscription().unwrap().unwrap().stage,
            SubscriptionStage::Active
        ));
        assert!(outbox.remove_unactivated_subscription().is_err());
        let path = directory.path().join("subscription.json");
        fs::write(&path, b"{}").unwrap();
        assert_eq!(
            outbox.load_subscription().err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
        fs::remove_file(&path).unwrap();
        symlink("outside", &path).unwrap();
        assert!(outbox.load_subscription().is_err());

        let fresh = tempfile::tempdir().unwrap();
        fs::set_permissions(fresh.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let outbox = Outbox::open(fresh.path()).unwrap();
        outbox.prepare_subscription(subscription()).unwrap();
        outbox.remove_unactivated_subscription().unwrap();
        assert!(outbox.load_subscription().unwrap().is_none());
    }
}
