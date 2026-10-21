//! Private, bounded service NDJSON sink. A full sink fails before MQTT ACK.
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{Mutex, OnceLock},
};

static SINK: OnceLock<Mutex<Sink>> = OnceLock::new();

struct Sink {
    file: File,
    maximum: u64,
    length: u64,
}

impl Sink {
    fn existing(mut file: File, maximum: u64) -> io::Result<Self> {
        let length = file.metadata()?.len();
        if length > maximum {
            return Err(io::Error::other("service output capacity exhausted"));
        }
        if length > 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last = [0_u8];
            file.read_exact(&mut last)?;
            if last != *b"\n" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "service output has an incomplete record",
                ));
            }
        }
        Ok(Self {
            file,
            maximum,
            length,
        })
    }

    fn append(&mut self, record: &str) -> io::Result<()> {
        let bytes = u64::try_from(record.len()).map_err(io::Error::other)? + 1;
        if self
            .length
            .checked_add(bytes)
            .is_none_or(|next| next > self.maximum)
        {
            return Err(io::Error::other("service output capacity exhausted"));
        }
        self.file.write_all(record.as_bytes())?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        self.file.sync_data()?;
        self.length += bytes;
        Ok(())
    }
}

pub fn configure(path: &Path, maximum: u64) -> io::Result<()> {
    let file = crate::windows_private::open_private_file(path, true)?;
    let sink = Sink::existing(file, maximum)?;
    SINK.set(Mutex::new(sink))
        .map_err(|_| io::Error::other("service output already initialized"))
}

pub fn active() -> bool {
    SINK.get().is_some()
}

pub fn append(record: &str) -> io::Result<()> {
    SINK.get()
        .ok_or_else(|| io::Error::other("service output unavailable"))?
        .lock()
        .map_err(|_| io::Error::other("service output unavailable"))?
        .append(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    #[test]
    fn capacity_never_truncates_records_and_reopen_rejects_partial_tail() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sink.ndjson");
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .append(true)
            .open(&path)
            .unwrap();
        let mut sink = Sink::existing(file, 8).unwrap();
        sink.append("{}").unwrap();
        assert!(sink.append("12345").is_err());
        drop(sink);
        assert_eq!(std::fs::read(&path).unwrap(), b"{}\n");
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&path)
            .unwrap();
        file.write_all(b"partial").unwrap();
        assert!(Sink::existing(file, 100).is_err());
    }
}
