use std::{io, path::Path};

#[cfg(any(target_os = "linux", windows))]
use std::io::Read;

#[cfg(any(target_os = "linux", windows))]
use super::cli;

#[cfg(any(target_os = "linux", windows))]
const MAX_PASSWORD_FILE_BYTES: u64 = 1026; // 1024 password bytes plus optional CRLF.

/// Reads a bounded secret from a private regular file without logging its contents.
///
/// On Linux/WSL, the opened file must belong to the process effective user and
/// have exactly mode 0600. Windows validates a private DACL on the opened handle.
pub fn read_password_file(path: &Path) -> io::Result<String> {
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "--password-file esta disponivel somente no Linux/WSL",
        ))
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.permissions().mode() & 0o7777 != 0o600
            || metadata.uid() != effective_uid()?
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "arquivo de senha deve ser regular, pertencer ao usuario atual e ter permissao 0600",
            ));
        }
        read_secret(file)
    }
    #[cfg(windows)]
    {
        read_secret(crate::windows_private::open_private_file(path, false)?)
    }
}

#[cfg(any(target_os = "linux", windows))]
fn read_secret(file: std::fs::File) -> io::Result<String> {
    let mut bytes = Vec::new();
    file.take(MAX_PASSWORD_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_PASSWORD_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arquivo de senha excede o limite de 1026 bytes",
        ));
    }
    let mut password = String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "arquivo de senha nao e UTF-8"))?;
    if password.ends_with('\n') {
        password.pop();
        if password.ends_with('\r') {
            password.pop();
        }
    }
    cli::validate_password(&password)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
    Ok(password)
}

#[cfg(target_os = "linux")]
pub(crate) fn effective_uid() -> io::Result<u32> {
    // Linux procfs exposes real, effective, saved and filesystem UIDs in order.
    let status = std::fs::read_to_string("/proc/self/status")?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|fields| fields.split_whitespace().nth(1))
        .and_then(|uid| uid.parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "UID efetivo indisponivel"))
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn windows_private_password_bounds_and_newline() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("password");
        std::fs::write(&path, b"secret\r\n").unwrap();
        let identity = crate::windows_private::current_sid().unwrap();
        let private = format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
        crate::windows_private::test_set_acl(directory.path(), &private);
        crate::windows_private::test_set_acl(&path, &private);
        assert_eq!(super::read_password_file(&path).unwrap(), "secret");
        for bytes in [
            vec![b'x'; 1027],
            vec![0xff],
            b"line\nbreak".to_vec(),
            Vec::new(),
        ] {
            std::fs::write(&path, bytes).unwrap();
            assert!(super::read_password_file(&path).is_err());
        }
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn accepts_private_file_and_optional_newline() {
        use super::read_password_file;
        use std::{fs, os::unix::fs::PermissionsExt};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("password");
        fs::write(&path, b"secret\r\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(read_password_file(&path).unwrap(), "secret");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_world_readable_file_and_oversized_secret() {
        use super::read_password_file;
        use std::{fs, os::unix::fs::PermissionsExt};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("password");
        fs::write(&path, b"secret\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_password_file(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, vec![b'x'; 1027]).unwrap();
        assert!(read_password_file(&path).is_err());
    }
}
