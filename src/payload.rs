// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
    time::Duration,
};
use tokio::{
    sync::mpsc,
    time::{Instant, timeout_at},
};

pub const MAX_BYTES: usize = 4096;
pub const INPUT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Text(String),
    File(PathBuf),
    Stdin,
}

pub struct Input {
    receiver: mpsc::Receiver<io::Result<Vec<u8>>>,
}
pub struct Publication {
    pub input: Input,
    pub first: Vec<u8>,
}

impl Input {
    pub fn start(source: Source, lines: bool) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel(1);
        if let Source::Text(text) = source {
            if lines {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--line-mode exige arquivo ou stdin",
                ));
            }
            validate_size(text.len())?;
            sender
                .try_send(Ok(text.into_bytes()))
                .map_err(|_| io::Error::other("fila de payload indisponivel"))?;
        } else {
            // A dedicated native thread is deliberately not Tokio's blocking pool:
            // an OS stdin read cannot be cancelled, but must not stall runtime shutdown.
            std::thread::Builder::new().name("mqtt-payload".into()).spawn(move || {
                let result = match source {
                    Source::File(path) => open_regular(&path).and_then(|file| feed(file, lines, &sender)),
                    Source::Stdin => feed(io::stdin().lock(), lines, &sender),
                    Source::Text(_) => unreachable!(),
                };
                if let Err(error) = result {
                    // Preserve category but never print an OS path or input bytes.
                    let message = if error.kind() == io::ErrorKind::InvalidData {
                        "payload excede 4096 bytes; mensagens anteriores podem estar confirmadas"
                    } else { "nao foi possivel ler entrada de payload" };
                    let _ = sender.blocking_send(Err(io::Error::new(error.kind(),message)));
                }
            })?;
        }
        Ok(Self { receiver })
    }

    pub async fn next(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.next_before(Instant::now() + INPUT_TIMEOUT).await
    }

    pub async fn next_before(&mut self, deadline: Instant) -> io::Result<Option<Vec<u8>>> {
        match timeout_at(deadline, self.receiver.recv()).await {
            Ok(Some(result)) => result.map(Some),
            Ok(None) => Ok(None),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "tempo esgotado aguardando entrada de payload",
            )),
        }
    }
}

fn open_regular(path: &std::path::Path) -> io::Result<File> {
    if !std::fs::metadata(path)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "entrada deve ser arquivo regular",
        ));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "entrada deve ser arquivo regular",
        ));
    }
    Ok(file)
}

fn validate_size(size: usize) -> io::Result<()> {
    if size > MAX_BYTES {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "payload excede 4096 bytes",
        ))
    } else {
        Ok(())
    }
}

fn raw(reader: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    validate_size(bytes.len())?;
    Ok(bytes)
}

fn line(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    let length = Read::take(&mut *reader, (MAX_BYTES + 3) as u64).read_until(b'\n', &mut bytes)?;
    if length == 0 {
        return Ok(None);
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    validate_size(bytes.len())?;
    Ok(Some(bytes))
}

fn feed(
    reader: impl Read,
    lines: bool,
    sender: &mpsc::Sender<io::Result<Vec<u8>>>,
) -> io::Result<()> {
    if lines {
        let mut reader = BufReader::with_capacity(MAX_BYTES + 3, reader);
        while !sender.is_closed() {
            let Some(bytes) = line(&mut reader)? else {
                break;
            };
            if sender.blocking_send(Ok(bytes)).is_err() {
                break;
            }
        }
    } else {
        let bytes = raw(reader)?;
        let _ = sender.blocking_send(Ok(bytes));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn raw_binary_empty_and_byte_limit() {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(raw(Cursor::new(&bytes)).unwrap(), bytes);
        assert_eq!(
            raw(Cursor::new(b"\xef\xbb\xbf\0\xff\r\n")).unwrap(),
            b"\xef\xbb\xbf\0\xff\r\n"
        );
        assert_eq!(raw(Cursor::new([])).unwrap(), [] as [u8; 0]);
        assert_eq!(raw(Cursor::new(vec![0; 4096])).unwrap().len(), 4096);
        assert!(raw(Cursor::new(vec![0; 4097])).is_err());
    }
    #[test]
    fn lines_frame_without_phantom_or_utf8_conversion() {
        let mut r = Cursor::new(b"\xef\xbb\xbf\xff\n\r\nlast\r");
        assert_eq!(line(&mut r).unwrap(), Some(b"\xef\xbb\xbf\xff".to_vec()));
        assert_eq!(line(&mut r).unwrap(), Some(Vec::new()));
        assert_eq!(line(&mut r).unwrap(), Some(b"last\r".to_vec()));
        assert_eq!(line(&mut r).unwrap(), None);
        let mut r = Cursor::new(b"a\n");
        assert_eq!(line(&mut r).unwrap(), Some(b"a".to_vec()));
        assert_eq!(line(&mut r).unwrap(), None);
        assert_eq!(line(&mut Cursor::new([])).unwrap(), None);
    }
    #[test]
    fn line_quota_excludes_only_lf_and_optional_cr() {
        for ending in [b"".as_slice(), b"\n", b"\r\n"] {
            let mut input = vec![b'x'; 4096];
            input.extend(ending);
            assert_eq!(line(&mut Cursor::new(input)).unwrap().unwrap().len(), 4096);
            let mut oversized = vec![b'x'; 4097];
            oversized.extend(ending);
            assert!(line(&mut Cursor::new(oversized)).is_err());
        }
    }
    #[tokio::test]
    async fn file_policy_and_error_redaction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private-name-never-echo");
        std::fs::write(&path, [0, 255, 10]).unwrap();
        let mut input = Input::start(Source::File(path), false).unwrap();
        assert_eq!(input.next().await.unwrap(), Some(vec![0, 255, 10]));
        assert_eq!(input.next().await.unwrap(), None);
        for path in [
            dir.path().to_path_buf(),
            dir.path().join("private-name-never-echo-missing"),
        ] {
            let mut input = Input::start(Source::File(path), false).unwrap();
            let error = input.next().await.unwrap_err().to_string();
            assert!(!error.contains("private-name"));
        }
    }
    #[tokio::test]
    async fn input_deadline_does_not_require_worker_to_finish() {
        let (_sender, receiver) = mpsc::channel(1);
        let mut input = Input { receiver };
        assert_eq!(
            input
                .next_before(Instant::now() + Duration::from_millis(10))
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
    }
}
