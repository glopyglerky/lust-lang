use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("lustr-doctor-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn program(root: &Path, name: &str) -> PathBuf {
    root.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

fn helper(scratch: &Scratch) {
    let source = scratch.0.join("recorder.rs");
    fs::write(
        &source,
        r#"
use std::io::Write;
fn main() {
    let executable = std::env::current_exe().unwrap();
    let name = executable.file_stem().unwrap().to_str().unwrap();
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut log = std::fs::OpenOptions::new().create(true).append(true)
        .open(std::env::var_os("DOCTOR_LOG").unwrap()).unwrap();
    writeln!(log, "{name}:{args:?}").unwrap();
    if name == "native-lust" {
        assert_eq!(args, ["--version"]);
        println!("lust fixture");
        return;
    }
    if name != "rustup" {
        eprintln!("unselected tool was invoked");
        std::process::exit(43);
    }
    assert_eq!(&args[..2], ["run", "--"]);
    assert!(std::env::var_os("CARGO").is_none());
    if args[2] == "missing-toolchain" {
        eprintln!("requested fixture toolchain is unavailable");
        std::process::exit(10);
    }
    match args[3].as_str() {
        "cargo" => { assert_eq!(&args[4..], ["--version"]); println!("cargo selected-fixture"); }
        "rustc" if args[4..] == ["--print", "sysroot"] => {
            println!("{}", std::env::var("DOCTOR_SYSROOT").unwrap());
        }
        "rustc" => {
            assert_eq!(&args[4..], ["--version", "--verbose"]);
            println!("rustc selected-fixture\nhost: fixture-host\nrelease: 1.99.1-nightly");
        }
        "rustdoc" => {
            assert_eq!(&args[4..], ["--version", "--verbose"]);
            println!("rustdoc selected-fixture");
        }
        _ => std::process::exit(44),
    }
}
"#,
    )
    .unwrap();
    let executable = program(&scratch.0, "rustup");
    let compiled = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc")))
        .args(["--edition=2024", "--crate-name=doctor_recorder"])
        .arg(source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    for name in ["cargo", "rustc", "rustdoc", "native-lust"] {
        fs::copy(&executable, program(&scratch.0, name)).unwrap();
    }
}

#[test]
fn doctor_probes_the_explicit_toolchain_without_installing_or_falling_back() {
    let scratch = Scratch::new();
    helper(&scratch);
    for name in ["selected-toolchain", "--install", "missing-toolchain"] {
        let log = scratch.0.join(format!("{name}.log"));
        let result = Command::new(env!("CARGO_BIN_EXE_lustr"))
            .args([&format!("+{name}"), "doctor"])
            .env("PATH", &scratch.0)
            .env("CARGO", program(&scratch.0, "cargo"))
            .env("LUSTC", program(&scratch.0, "native-lust"))
            .env("DOCTOR_LOG", &log)
            .env("DOCTOR_SYSROOT", &scratch.0)
            .env_remove("LUST_CARGO")
            .env_remove("RUSTC")
            .env_remove("RUSTDOC")
            .env_remove("RUSTUP_TOOLCHAIN")
            .env_remove("LUST_CRANELIFT_BACKEND")
            .output()
            .unwrap();
        let output = String::from_utf8(result.stdout).unwrap();
        let observed = fs::read_to_string(log).unwrap();
        assert_eq!(result.status.code(), Some(i32::from(name == "missing-toolchain")), "{output}\n{observed}");
        assert_eq!(observed.lines().filter(|line| line.starts_with("rustup:")).count(), 4, "{observed}");
        assert_eq!(observed.lines().filter(|line| line.starts_with("native-lust:")).count(), 1, "{observed}");
        assert_eq!(observed.lines().count(), 5, "unexpected direct compiler or installation command: {observed}");
        for (tool, flags) in [
            ("cargo", "\"--version\""),
            ("rustc", "\"--version\", \"--verbose\""),
            ("rustdoc", "\"--version\", \"--verbose\""),
            ("rustc", "\"--print\", \"sysroot\""),
        ] {
            assert!(observed.contains(&format!("rustup:[\"run\", \"--\", \"{name}\", \"{tool}\", {flags}]")), "{observed}");
        }
        assert!(
            output.contains(&format!("\"program\":\"rustup\",\"args\":[\"run\",\"--\",\"{name}\",\"cargo\",\"--version\"]")),
            "{output}"
        );
        assert!(output.contains("\"sysroot\":{"), "{output}");
        assert!(!output.contains("unselected tool was invoked"), "{output}");
    }
    let conflict_log = scratch.0.join("conflict.log");
    let conflict = Command::new(env!("CARGO_BIN_EXE_lustr"))
        .args(["+selected-toolchain", "doctor"])
        .env("LUST_CARGO", program(&scratch.0, "cargo"))
        .env("DOCTOR_LOG", &conflict_log)
        .env("PATH", &scratch.0)
        .output()
        .unwrap();
    assert_eq!(conflict.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("+toolchain cannot be combined with LUST_CARGO"));
    assert!(!conflict_log.exists(), "conflicting selectors must fail before any probe");
}
