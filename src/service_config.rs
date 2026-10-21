//! Bounded service launch configuration, without mutating the environment.
use serde::Deserialize;
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    version: u32,
    pub arguments: Vec<String>,
    #[serde(default)]
    pub output_file: Option<PathBuf>,
    #[serde(default = "default_output_max_bytes")]
    pub output_max_bytes: u64,
}

fn default_output_max_bytes() -> u64 {
    8 * 1024 * 1024
}

pub fn load(path: &Path) -> io::Result<Document> {
    if !path.is_absolute() {
        return Err(invalid());
    }
    #[cfg(windows)]
    let file = crate::windows_private::open_private_file(path, false)?;
    #[cfg(not(windows))]
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid());
    }
    let mut text = String::new();
    file.take(65_537).read_to_string(&mut text)?;
    if text.len() > 65_536 {
        return Err(invalid());
    }
    let document: Document = toml::from_str(&text).map_err(|_| invalid())?;
    if document.version != 1
        || document.arguments.len() > 256
        || document
            .arguments
            .iter()
            .any(|arg| arg.contains('\0') || arg == "--profile")
        || document
            .arguments
            .windows(2)
            .any(|pair| pair[0] == "--will-message-file" && !Path::new(&pair[1]).is_absolute())
        || !(4096..=64 * 1024 * 1024).contains(&document.output_max_bytes)
        || document
            .output_file
            .as_ref()
            .is_some_and(|path| !path.is_absolute())
        || !document
            .arguments
            .first()
            .is_some_and(|arg| arg == "pub" || arg == "sub")
    {
        return Err(invalid());
    }
    Ok(document)
}

pub fn validate(cli: &crate::cli::Cli, has_output: bool) -> io::Result<()> {
    if [&cli.ca, &cli.cert, &cli.key]
        .iter()
        .any(|path| path.as_ref().is_some_and(|path| !path.is_absolute()))
    {
        return Err(invalid());
    }
    match &cli.mode {
        crate::cli::Mode::Pub {
            message: crate::payload::Source::File(path),
            ..
        } if path.is_absolute() && !has_output => {}
        crate::cli::Mode::Sub { .. }
            if has_output && cli.output == crate::output::Output::Jsonl => {}
        _ => return Err(invalid()),
    }
    if cli.transport_mode != crate::cli::TransportMode::OpenLab
        && !cli
            .password_file
            .as_ref()
            .is_some_and(|path| path.is_absolute())
    {
        return Err(invalid());
    }
    Ok(())
}

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid or unsupported service configuration",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_rejects_unknown_fields_oversize_and_non_client_commands() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("service.toml");
        #[cfg(windows)]
        {
            std::fs::write(&path, b"").unwrap();
            let identity = crate::windows_private::current_sid().unwrap();
            let private = format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
            crate::windows_private::test_set_acl(dir.path(), &private);
            crate::windows_private::test_set_acl(&path, &private);
        }
        for text in [
            "version=1\narguments=['--version']",
            "version=2\narguments=['pub']",
            "version=1\narguments=['pub']\npassword='secret'",
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(load(&path).is_err());
        }
        std::fs::write(&path, " ".repeat(65_537)).unwrap();
        assert!(load(&path).is_err());
        std::fs::write(&path, "version=1\narguments=['pub','--open-lab']").unwrap();
        #[cfg(windows)]
        {
            let identity = crate::windows_private::current_sid().unwrap();
            let private = format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
            crate::windows_private::test_set_acl(dir.path(), &private);
            crate::windows_private::test_set_acl(&path, &private);
        }
        assert_eq!(load(&path).unwrap().arguments, ["pub", "--open-lab"]);
        assert!(load(Path::new("relative.toml")).is_err());
    }
    #[test]
    fn service_never_prompts_or_discards_subscriber_output() {
        for args in [
            vec!["sub", "--open-lab", "--topic", "test"],
            vec!["pub", "--open-lab", "--topic", "test", "--stdin"],
            vec!["pub", "--open-lab", "--topic", "test", "--message", "test"],
        ] {
            let cli = crate::cli::parse(args.into_iter().map(str::to_owned)).unwrap();
            assert!(validate(&cli, false).is_err());
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("message.bin");
        let cli = crate::cli::parse(
            [
                "pub",
                "--open-lab",
                "--topic",
                "test",
                "--message-file",
                path.to_str().unwrap(),
            ]
            .map(str::to_owned),
        )
        .unwrap();
        assert!(validate(&cli, false).is_ok());
    }

    #[test]
    fn subscriber_requires_jsonl_sink_and_bounded_absolute_configuration() {
        let cli = crate::cli::parse(
            ["sub", "--open-lab", "--topic", "test", "--output", "jsonl"].map(str::to_owned),
        )
        .unwrap();
        assert!(validate(&cli, true).is_ok());
        assert!(validate(&cli, false).is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("service.toml");
        #[cfg(windows)]
        {
            std::fs::write(&path, b"").unwrap();
            let identity = crate::windows_private::current_sid().unwrap();
            let private = format!("D:P(A;OICI;FA;;;{identity})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
            crate::windows_private::test_set_acl(dir.path(), &private);
            crate::windows_private::test_set_acl(&path, &private);
        }
        for fields in [
            "output_file='relative'",
            "output_max_bytes=4095",
            "output_max_bytes=67108865",
        ] {
            std::fs::write(&path, format!("version=1\narguments=['sub']\n{fields}\n")).unwrap();
            assert!(load(&path).is_err());
        }
    }
}
