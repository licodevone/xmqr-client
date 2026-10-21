// SPDX-License-Identifier: MIT
//! Private NTFS files. ACLs are checked on open handles, never inferred from readonly.
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, Prefix},
};
use windows_permissions::{
    SecurityDescriptor,
    constants::{AceType, SeObjectType, SecurityInformation},
    wrappers,
};

const REPARSE: u32 = 0x400;
const OPEN_REPARSE: u32 = 0x0020_0000;
const BACKUP_SEMANTICS: u32 = 0x0200_0000;
const READ_CONTROL: u32 = 0x0002_0000;

fn denied() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "private Windows file requires a trusted owner, restrictive DACL and no reparse points",
    )
}

fn trusted(sid: &str, current: &str) -> bool {
    sid == current || matches!(sid, "S-1-5-18" | "S-1-5-19" | "S-1-5-32-544")
}

pub fn validate(file: &File) -> io::Result<()> {
    let identity = current_sid()?;
    let metadata = file.metadata()?;
    if metadata.file_attributes() & REPARSE != 0 {
        return Err(denied());
    }
    let sd = wrappers::GetSecurityInfo(
        file,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Owner | SecurityInformation::Dacl,
    )?;
    validate_descriptor(&sd, &identity)
}

fn validate_descriptor(sd: &SecurityDescriptor, identity: &str) -> io::Result<()> {
    if !sd
        .owner()
        .is_some_and(|owner| trusted(&owner.to_string(), identity))
    {
        return Err(denied());
    }
    let acl = sd.dacl().ok_or_else(denied)?;
    if acl.len() == 0 {
        return Err(denied());
    }
    for index in 0..acl.len() {
        let ace = acl.get_ace(index).ok_or_else(denied)?;
        // Unknown, callback and object ACE semantics fail closed. Broad grants
        // are rejected even when a deny ACE might currently mask them.
        match ace.ace_type() {
            AceType::ACCESS_DENIED_ACE_TYPE => {}
            AceType::ACCESS_ALLOWED_ACE_TYPE
                if ace
                    .sid()
                    .is_some_and(|sid| trusted(&sid.to_string(), identity)) => {}
            _ => return Err(denied()),
        }
    }
    Ok(())
}

fn ancestors(path: &Path) -> io::Result<Vec<File>> {
    if !path.is_absolute()
        || !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        || path.components().any(|c| match c {
            Component::ParentDir | Component::CurDir => true,
            Component::Normal(name) => {
                let name = name.to_string_lossy();
                let base = name
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                name.contains(':')
                    || name.ends_with(['.', ' '])
                    || matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (base.len() == 4
                        && (base.starts_with("COM") || base.starts_with("LPT"))
                        && base.as_bytes()[3].is_ascii_digit())
            }
            _ => false,
        })
    {
        return Err(denied());
    }
    let mut chain: Vec<_> = path.parent().ok_or_else(denied)?.ancestors().collect();
    chain.reverse();
    let mut handles = Vec::new();
    for ancestor in chain {
        let handle = OpenOptions::new()
            .access_mode(READ_CONTROL)
            .share_mode(3)
            .custom_flags(BACKUP_SEMANTICS | OPEN_REPARSE)
            .open(ancestor)?;
        let metadata = handle.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & REPARSE != 0 {
            return Err(denied());
        }
        handles.push(handle);
    }
    validate(handles.last().ok_or_else(denied)?)?;
    Ok(handles)
}

pub fn open_private_file(path: &Path, write: bool) -> io::Result<File> {
    let _parents = ancestors(path)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(write)
        .share_mode(1)
        .custom_flags(OPEN_REPARSE);
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(denied());
    }
    validate(&file)?;
    Ok(file)
}

pub fn current_sid() -> io::Result<String> {
    use winsafe::{HPROCESS, TokenInfo, co};
    let token = HPROCESS::GetCurrentProcess()
        .OpenProcessToken(co::TOKEN::QUERY)
        .map_err(io::Error::other)?;
    match token
        .GetTokenInformation(co::TOKEN_INFORMATION_CLASS::User)
        .map_err(io::Error::other)?
    {
        TokenInfo::User(user) => user
            .User
            .Sid()
            .map(|sid| sid.to_string())
            .ok_or_else(denied),
        _ => Err(denied()),
    }
}

#[cfg(test)]
pub(crate) fn test_set_acl(path: &Path, sddl: &str) {
    use windows_permissions::{LocalBox, constants::SeObjectType, wrappers};
    let descriptor: LocalBox<SecurityDescriptor> = sddl.parse().unwrap();
    let mut handle = OpenOptions::new()
        .access_mode(READ_CONTROL | 0x0004_0000)
        .share_mode(3)
        .custom_flags(BACKUP_SEMANTICS | OPEN_REPARSE)
        .open(path)
        .unwrap();
    wrappers::SetSecurityInfo(
        &mut handle,
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        descriptor.dacl(),
        None,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_empty_untrusted_owner_and_masked_public_grants_fail_closed() {
        use windows_permissions::LocalBox;
        let identity = current_sid().unwrap();
        for text in [
            format!("O:{identity}D:"),
            format!("O:{identity}"),
            format!("O:{identity}D:(D;;FA;;;WD)(A;;FR;;;WD)"),
            "O:S-1-5-21-1-2-3-1000D:(A;;FA;;;SY)".into(),
        ] {
            let descriptor: LocalBox<SecurityDescriptor> = text.parse().unwrap();
            assert!(validate_descriptor(&descriptor, &identity).is_err());
        }
    }
    #[test]
    fn real_acl_rejects_broad_file_and_parent_grants() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("private");
        std::fs::write(&path, b"secret\n").unwrap();
        let identity = current_sid().unwrap();
        let private = format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
        test_set_acl(directory.path(), &private);
        test_set_acl(&path, &private);
        assert!(open_private_file(&path, false).is_ok());
        for name in ["private:stream", "private.", "private ", "NUL", "CON.txt"] {
            assert!(open_private_file(&directory.path().join(name), false).is_err());
        }
        for sid in ["WD", "AU", "BU"] {
            test_set_acl(
                &path,
                &format!("D:P(A;OICI;FA;;;{identity})(A;;FR;;;{sid})"),
            );
            assert!(open_private_file(&path, false).is_err(), "broad file {sid}");
            test_set_acl(&path, &private);
            test_set_acl(
                directory.path(),
                &format!("D:P(A;OICI;FA;;;{identity})(A;;FR;;;{sid})"),
            );
            assert!(
                open_private_file(&path, false).is_err(),
                "broad parent {sid}"
            );
            test_set_acl(directory.path(), &private);
        }
    }
    #[test]
    fn real_junction_ancestor_is_rejected() {
        let junction = std::env::var_os("XMQR_TEST_REPARSE_DIRECTORY").map_or_else(
            || std::path::PathBuf::from(r"C:\Documents and Settings"),
            std::path::PathBuf::from,
        );
        assert_ne!(
            std::fs::symlink_metadata(&junction)
                .unwrap()
                .file_attributes()
                & REPARSE,
            0,
            "fixture must be a real Windows reparse directory"
        );
        let error = open_private_file(&junction.join("xmqr-c151-never-opened"), false).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), denied().to_string());
    }
}
