//! Exercise the installed command as a process, with a Rust executable standing
//! in for Cargo/Lust/rustup. It reports argument bytes and inherited settings;
//! successful return from a mocked build is never compiler qualification.
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("cargo-lust-process-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn record(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn helper(scratch: &Scratch) -> PathBuf {
    let source = scratch.0.join("recorder.rs");
    fs::write(&source, r#"
use std::io::{Read, Write};
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|byte| format!("{byte:02x}")).collect() }
fn main() {
    let record = std::env::var_os("RECORDER_LOG").unwrap();
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(record).unwrap();
    writeln!(file, "invocation").unwrap();
    for argument in std::env::args_os().skip(1) {
        writeln!(file, "arg={}", hex(argument.as_encoded_bytes())).unwrap();
    }
    for key in ["CARGO", "RUSTC", "RUSTDOC", "RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_PROFILE_DEV_CODEGEN_BACKEND", "CARGO_PROFILE_TEST_CODEGEN_BACKEND"] {
        let value = std::env::var_os(key).map(|value| hex(value.as_encoded_bytes())).unwrap_or_else(|| "<unset>".to_owned());
        writeln!(file, "env {key}={value}").unwrap();
    }
    file.flush().unwrap();
    std::io::stdout().write_all(b"{\"diagnostic\":\"fixture\"}\n\xff").unwrap();
    std::io::stderr().write_all(b"fixture diagnostic\n\xfe").unwrap();
    if std::env::var_os("RECORDER_STDIN").is_some() {
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input).unwrap();
        std::io::stdout().write_all(&input).unwrap();
    }
    #[cfg(unix)]
    if std::env::var_os("RECORDER_SIGNAL").is_some() {
        unsafe extern "C" { fn raise(signal: i32) -> i32; }
        unsafe { raise(15); }
    }
    std::process::exit(23);
}
"#).unwrap();
    let executable = scratch.0.join(format!("cargo recorder{}", std::env::consts::EXE_SUFFIX));
    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc")))
        .args(["--edition=2024", "--crate-name", "cargo_lust_recorder"])
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(output.status.success(), "failed to compile process fixture: {}", String::from_utf8_lossy(&output.stderr));
    executable
}

fn coordinator(executable: &Path, log: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-lust"));
    command.env("LUST_CARGO", executable).env("RECORDER_LOG", log);
    command.env_remove("CARGO_PROFILE_DEV_CODEGEN_BACKEND").env_remove("CARGO_PROFILE_TEST_CODEGEN_BACKEND");
    command.env_remove("LUST_CRANELIFT_BACKEND");
    command.env_remove("RECORDER_STDIN").env_remove("RECORDER_SIGNAL");
    command
}

#[test]
fn process_contract_preserves_arguments_io_failure_and_toolchain_routing() {
    let scratch = Scratch::new();
    let executable = helper(&scratch);
    let log = scratch.0.join("cargo.log");
    let arguments = ["run", "--manifest-path", "directory with spaces/Cargo.toml", "--", "--engine", "lust", "--profile", "prod"];
    let mut child = coordinator(&executable, &log)
        .args(["lust", "--engine", "rustc"])
        .args(arguments)
        .env("LUSTC", "") // An unrelated empty override must not break Cargo.
        .env("RUSTC", "selected compiler")
        .env("RUSTDOC", "selected rustdoc")
        .env("RUSTFLAGS", "-C panic=abort -C opt-level=2")
        .env("RUSTC_WRAPPER", "existing wrapper")
        .env("RUSTC_WORKSPACE_WRAPPER", "workspace wrapper")
        .env("RECORDER_STDIN", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"stdin bytes\0\xff").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(23), "exit status must come from the child");
    assert_eq!(output.stdout, b"{\"diagnostic\":\"fixture\"}\n\xffstdin bytes\0\xff");
    assert_eq!(output.stderr, b"fixture diagnostic\n\xfe");
    let observed = record(&log);
    assert_eq!(observed.lines().filter(|line| *line == "invocation").count(), 1, "failed program was retried");
    let received: Vec<_> = observed.lines().filter_map(|line| line.strip_prefix("arg=")).collect();
    let expected: Vec<_> = arguments.iter().map(|arg| hex(arg.as_bytes())).collect();
    assert_eq!(received, expected);
    for (key, value) in [
        ("RUSTC", "selected compiler"),
        ("RUSTDOC", "selected rustdoc"),
        ("RUSTFLAGS", "-C panic=abort -C opt-level=2"),
        ("RUSTC_WRAPPER", "existing wrapper"),
        ("RUSTC_WORKSPACE_WRAPPER", "workspace wrapper"),
    ] {
        assert!(observed.contains(&format!("env {key}={}\n", hex(value.as_bytes()))));
    }

    // A failed Cranelift build is returned once; it never triggers a Cargo or
    // Lust retry. Only the backend profile settings differ from the input.
    let backend = scratch.0.join("mock backend with spaces.so");
    fs::write(&backend, "process fixture only; never loaded as a backend").unwrap();
    let fast_log = scratch.0.join("cranelift.log");
    let fast = coordinator(&executable, &fast_log)
        .env("LUST_CRANELIFT_BACKEND", &backend)
        .args(["--engine", "cranelift", "build"])
        .output()
        .unwrap();
    assert_eq!(fast.status.code(), Some(23));
    assert_eq!(fast.stdout, b"{\"diagnostic\":\"fixture\"}\n\xff");
    assert!(fast.stderr.ends_with(b"fixture diagnostic\n\xfe"));
    let observed = record(&fast_log);
    assert_eq!(observed.lines().filter(|line| *line == "invocation").count(), 1);
    for key in ["CARGO_PROFILE_DEV_CODEGEN_BACKEND", "CARGO_PROFILE_TEST_CODEGEN_BACKEND"] {
        assert!(observed.contains(&format!("env {key}={}\n", hex(backend.as_os_str().as_encoded_bytes()))));
    }

    let release_log = scratch.0.join("release.log");
    let release = coordinator(&executable, &release_log).args(["--engine", "cranelift", "build", "-vr"]).output().unwrap();
    assert_eq!(release.status.code(), Some(23));
    assert_eq!(release.stderr, b"fixture diagnostic\n\xfe");
    let observed = record(&release_log);
    assert!(!observed.contains(&format!("arg={}\n", hex(b"-Zcodegen-backend"))));
    assert!(observed.contains("env CARGO_PROFILE_DEV_CODEGEN_BACKEND=<unset>"));

    let lust_log = scratch.0.join("lust.log");
    let lust = coordinator(&executable, &lust_log)
        .env("LUST_CARGO", "") // Irrelevant for this route.
        .env("LUSTC", &executable)
        .args(["--engine", "lust", "check", "source with spaces.rs", "--json"])
        .output()
        .unwrap();
    assert_eq!(lust.status.code(), Some(23));
    assert_eq!(lust.stderr, b"fixture diagnostic\n\xfe");
    assert_eq!(record(&lust_log).lines().filter(|line| *line == "invocation").count(), 1);

    let dry_log = scratch.0.join("plan.log");
    let dry = coordinator(&executable, &dry_log)
        .env("LUST_CRANELIFT_BACKEND", &backend)
        .args(["--engine=cranelift", "plan", "build"])
        .output()
        .unwrap();
    assert!(dry.status.success());
    assert!(dry.stderr.is_empty());
    assert!(String::from_utf8(dry.stdout).unwrap().contains("\"selected_engine\":\"cranelift\""));
    assert!(!dry_log.exists(), "a plan executed the project tool");

    // Missing configuration or a named stock backend cannot quietly compile
    // with broken unwind behavior. Neither execution nor planning starts Cargo.
    for (index, backend) in [None, Some(""), Some("cranelift"), Some("relative/backend.so")].into_iter().enumerate() {
        for command in [vec!["build"], vec!["plan", "build"]] {
            let rejected_log = scratch.0.join(format!("rejected-backend-{index}.log"));
            let mut rejected = coordinator(&executable, &rejected_log);
            if let Some(backend) = backend {
                rejected.env("LUST_CRANELIFT_BACKEND", backend);
            }
            let rejected = rejected.args(["--engine", "cranelift"]).args(command).output().unwrap();
            assert_eq!(rejected.status.code(), Some(2));
            assert!(rejected.stdout.is_empty());
            assert!(String::from_utf8_lossy(&rejected.stderr).contains("LUST_CRANELIFT_BACKEND"));
            assert!(!rejected_log.exists(), "invalid backend configuration executed Cargo");
        }
    }

    let absent_log = scratch.0.join("absent.log");
    let absent = coordinator(&scratch.0.join("does not exist"), &absent_log).arg("build").output().unwrap();
    assert_eq!(absent.status.code(), Some(127));
    assert!(absent.stdout.is_empty());
    assert!(!absent_log.exists());
    assert!(String::from_utf8_lossy(&absent.stderr).contains("cannot execute"));

    // An outer `cargo lust` exports CARGO as an absolute toolchain Cargo, which
    // cannot parse +name. The coordinator must use rustup's run command instead.
    let rustup = scratch.0.join(format!("rustup{}", std::env::consts::EXE_SUFFIX));
    fs::copy(&executable, &rustup).unwrap();
    let rustup_log = scratch.0.join("rustup.log");
    let explicit = coordinator(&executable, &rustup_log)
        .env_remove("LUST_CARGO")
        .env("CARGO", scratch.0.join("stale absolute cargo"))
        .env("PATH", &scratch.0)
        .args(["+selected-nightly", "build"])
        .output()
        .unwrap();
    assert_eq!(explicit.status.code(), Some(23), "explicit toolchain did not reach rustup: {}", String::from_utf8_lossy(&explicit.stderr));
    let observed = record(&rustup_log);
    let received: Vec<_> = observed.lines().filter_map(|line| line.strip_prefix("arg=")).collect();
    let expected: Vec<_> = ["run", "--", "selected-nightly", "cargo", "build"].iter().map(|arg| hex(arg.as_bytes())).collect();
    assert_eq!(received, expected);
    assert!(observed.contains("env CARGO=<unset>\n"));

    let conflict_log = scratch.0.join("conflict.log");
    let conflict = coordinator(&executable, &conflict_log).args(["+nightly", "build"]).output().unwrap();
    assert_eq!(conflict.status.code(), Some(2));
    assert!(!conflict_log.exists(), "conflicting toolchain/executable choices executed a tool");

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        use std::os::unix::process::ExitStatusExt;
        let native_path = scratch.0.join(OsString::from_vec(b"cargo-\xff".to_vec()));
        fs::copy(&executable, &native_path).unwrap();
        let native_log = scratch.0.join("native.log");
        let native = coordinator(&native_path, &native_log)
            .args(["build", "--manifest-path"])
            .arg(OsString::from_vec(b"file-\xff/Cargo.toml".to_vec()))
            .output()
            .unwrap();
        assert_eq!(native.status.code(), Some(23));
        assert!(record(&native_log).contains(&format!("arg={}\n", hex(b"file-\xff/Cargo.toml"))));

        let signal_log = scratch.0.join("signal.log");
        let signal = coordinator(&executable, &signal_log).env("RECORDER_SIGNAL", "1").arg("run").output().unwrap();
        assert_eq!(signal.status.signal(), Some(15), "coordinator concealed child signal termination");
        assert_eq!(record(&signal_log).lines().filter(|line| *line == "invocation").count(), 1);
    }
}

#[test]
fn cargo_alias_can_select_release_without_enabling_cranelift() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join(".cargo")).unwrap();
    fs::create_dir_all(scratch.0.join("src")).unwrap();
    fs::write(scratch.0.join("Cargo.toml"), "[package]\nname = \"alias-proof\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\n")
        .unwrap();
    fs::write(scratch.0.join(".cargo/config.toml"), "[alias]\nb = \"build --release\"\n").unwrap();
    fs::write(scratch.0.join("src/main.rs"), "fn main() { println!(\"debug={}\", cfg!(debug_assertions)); }\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
        .current_dir(&scratch.0)
        .env("LUST_CARGO", env!("CARGO"))
        .env("CARGO_TARGET_DIR", scratch.0.join("target"))
        .env_remove("CARGO_PROFILE_DEV_CODEGEN_BACKEND")
        .env_remove("CARGO_PROFILE_TEST_CODEGEN_BACKEND")
        .args(["--engine", "cranelift", "b", "--offline", "--quiet"])
        .output()
        .unwrap();
    assert!(output.status.success(), "Cargo alias was routed through a development backend: {}", String::from_utf8_lossy(&output.stderr));
    let executable = scratch.0.join("target/release").join(format!("alias-proof{}", std::env::consts::EXE_SUFFIX));
    let ran = Command::new(executable).output().unwrap();
    assert!(ran.status.success());
    assert_eq!(ran.stdout, b"debug=false\n");
}
