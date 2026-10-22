use std::{env, fs, path::PathBuf, process::Command};

fn resource_compiler() -> PathBuf {
    if let Some(compiler) = env::var_os("RC") {
        return compiler.into();
    }
    if let Some(path) = env::var_os("PATH") {
        for directory in env::split_paths(&path) {
            let compiler = directory.join("rc.exe");
            if compiler.is_file() {
                return compiler;
            }
        }
    }
    if let (Some(sdk), Some(version)) = (
        env::var_os("WindowsSdkDir"),
        env::var_os("WindowsSDKVersion"),
    ) {
        let compiler = PathBuf::from(sdk)
            .join("bin")
            .join(version)
            .join("x64")
            .join("rc.exe");
        if compiler.is_file() {
            return compiler;
        }
    }
    let compiler =
        PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\rc.exe");
    assert!(
        compiler.is_file(),
        "Windows SDK resource compiler not found; set RC to the full rc.exe path"
    );
    compiler
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/client.ico");
    for variable in ["RC", "PATH", "WindowsSdkDir", "WindowsSDKVersion"] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ENV").as_deref(),
        Ok("msvc"),
        "Windows service packaging requires an MSVC target"
    );
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"));
    let icon = manifest.join("assets/client.ico");
    let icon_path = icon.to_str().expect("icon path must be Unicode");
    assert!(!icon_path.contains(['"', '\n', '\r']), "invalid icon path");
    let script = output.join("client.rc");
    let resource = output.join("client.res");
    fs::write(
        &script,
        format!("1 ICON \"{}\"\n", icon_path.replace('\\', "/")),
    )
    .expect("write Client icon resource script");
    let status = Command::new(resource_compiler())
        .args(["/nologo", "/c65001", "/fo"])
        .arg(&resource)
        .arg(&script)
        .status()
        .expect("execute Windows SDK resource compiler");
    assert!(status.success(), "Client icon resource compilation failed");
    println!(
        "cargo:rustc-link-arg-bin=mqtt-client={}",
        resource.display()
    );
}
