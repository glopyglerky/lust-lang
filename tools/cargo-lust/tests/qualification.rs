//! End-to-end qualification of the command that users install.
//!
//! These fixtures use only path dependencies. Nothing is downloaded and every run
//! owns its copied source and target directories. Optional backend tests are
//! ignored explicitly, so an unavailable backend never counts as verified.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED: &str = "derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1\n";
const EXPECTED_PANIC_TRACE: &str =
    "fixture: entering catch_unwind\nfixture: panic hook: qualification panic\nfixture: catch_unwind returned; caught=true drops=1\n";
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("cargo-lust-{name}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn fixture(&self, name: &str) -> PathBuf {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(name);
        // Spaces deliberately exercise argument preservation.
        let destination = self.0.join(format!("{name} with spaces"));
        copy_tree(&source, &destination);
        destination
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = destination.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink(), "fixture must not contain symlinks");
        if kind.is_dir() {
            assert_ne!(entry.file_name(), OsStr::new("target"), "fixture has build output");
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn cargo() -> OsString {
    std::env::var_os("LUST_TEST_CARGO").unwrap_or_else(|| OsString::from(env!("CARGO")))
}

fn command(scratch: &Scratch, fixture: &Path, target: &str, engine: Option<&str>) -> Command {
    let mut command = match engine {
        None => Command::new(cargo()),
        Some(engine) => {
            let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-lust"));
            command.env("LUST_CARGO", cargo());
            if engine != "default" {
                command.args(["--engine", engine]);
            }
            command
        }
    };
    command.current_dir(fixture);
    command.env("CARGO_TARGET_DIR", scratch.0.join(target));
    // The test executable may itself have been built with the complementary
    // backend. Fixture baselines must still use ordinary Cargo; explicit
    // Cranelift invocations set their own backend through the coordinator.
    command.env_remove("CARGO_PROFILE_DEV_CODEGEN_BACKEND");
    command.env_remove("CARGO_PROFILE_TEST_CODEGEN_BACKEND");
    command
}

fn execute(command: &mut Command) -> Output {
    command.output().unwrap_or_else(|error| panic!("could not execute {command:?}: {error}"))
}

fn success(output: &Output) {
    success_at("command", output);
}

fn success_at(stage: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{stage} failed: status={}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn full_rust_workspace_runs_identically_through_default_and_rustc_engines() {
    let scratch = Scratch::new("full-rust");
    let fixture = scratch.fixture("full-rust");
    let arguments =
        ["run", "--offline", "--locked", "--quiet", "-p", "fixture-app", "--", "--release", "two words", "--profile=application-argument"];
    let baseline = execute(command(&scratch, &fixture, "baseline", None).args(arguments));
    success(&baseline);
    let expected = format!("{EXPECTED}argument=--release\nargument=two words\nargument=--profile=application-argument\n");
    assert_eq!(baseline.stdout, expected.as_bytes());

    for engine in ["default", "rustc"] {
        let candidate = execute(command(&scratch, &fixture, engine, Some(engine)).args(arguments));
        success(&candidate);
        assert_eq!(candidate.stdout, baseline.stdout, "output differs for {engine}");
        assert_eq!(candidate.status.code(), baseline.status.code());
    }
}

#[test]
fn full_workspace_check_test_and_doctest_remain_cargo_operations() {
    let scratch = Scratch::new("commands");
    let fixture = scratch.fixture("full-rust");
    let checked = execute(command(&scratch, &fixture, "candidate", Some("rustc")).args([
        "check",
        "--offline",
        "--locked",
        "--workspace",
        "--message-format=json",
    ]));
    success(&checked);
    let stdout = String::from_utf8(checked.stdout).unwrap();
    assert!(stdout.contains("build-finished"));
    assert!(stdout.lines().all(|line| line.starts_with('{')), "coordinator polluted Cargo JSON: {stdout}");

    let tested = execute(command(&scratch, &fixture, "candidate", Some("rustc")).args(["test", "--offline", "--locked", "--workspace"]));
    success(&tested);
    let stdout = String::from_utf8_lossy(&tested.stdout);
    assert!(stdout.contains("features_keep_their_rust_semantics"), "library test was not run: {stdout}");
    assert!(stdout.contains("body_probe"), "doctest was not run: {stdout}");
}

#[test]
fn borrow_errors_have_the_same_failure_status_and_are_not_laundered() {
    let scratch = Scratch::new("invalid");
    let fixture = scratch.fixture("invalid-borrow");
    let baseline = execute(command(&scratch, &fixture, "baseline", None).args(["check", "--offline", "--locked"]));
    assert!(!baseline.status.success());
    assert!(String::from_utf8_lossy(&baseline.stderr).contains("E0502"));
    for engine in ["default", "rustc"] {
        let rejected = execute(command(&scratch, &fixture, engine, Some(engine)).args(["check", "--offline", "--locked"]));
        assert!(!rejected.status.success());
        assert_eq!(rejected.status.code(), baseline.status.code());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("E0502"));
    }
}

#[test]
fn manifest_paths_and_cargo_subcommand_invocation_preserve_arguments() {
    let scratch = Scratch::new("manifest");
    let fixture = scratch.fixture("full-rust");
    let output = execute(
        Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
            .current_dir(&scratch.0)
            .env("LUST_CARGO", cargo())
            .env("CARGO_TARGET_DIR", scratch.0.join("manifest-target"))
            .env_remove("CARGO_PROFILE_DEV_CODEGEN_BACKEND")
            .env_remove("CARGO_PROFILE_TEST_CODEGEN_BACKEND")
            .args(["lust", "--engine", "rustc", "build", "--offline", "--locked", "--manifest-path"])
            .arg(fixture.join("Cargo.toml")),
    );
    success(&output);
    let executable = scratch.0.join("manifest-target/debug").join(format!("fixture-app{}", std::env::consts::EXE_SUFFIX));
    let executed = execute(&mut Command::new(executable));
    success(&executed);
    assert_eq!(executed.stdout, EXPECTED.as_bytes());
}

#[test]
#[ignore = "requires a built Lust compiler; set LUST_TEST_LUST to its executable"]
fn actual_lust_accepts_its_subset_and_keeps_unsupported_rust_rejected() {
    let lust = std::env::var_os("LUST_TEST_LUST").expect("set LUST_TEST_LUST before running this qualification");
    let scratch = Scratch::new("actual-lust");
    let fixture = scratch.fixture("subset");
    let accepted = execute(
        Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
            .current_dir(&fixture)
            .env("LUSTC", &lust)
            .env("LUST_NO_DAEMON", "1")
            .args(["--engine", "lust", "run"])
            .arg(fixture.join("supported.rs")),
    );
    success(&accepted);
    assert_eq!(accepted.stdout, b"42\n");

    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let rustc_accepts = execute(
        Command::new(rustc)
            .current_dir(&fixture)
            .args(["--edition=2024", "--emit=metadata"])
            .arg(fixture.join("unsupported.rs"))
            .arg("-o")
            .arg(scratch.0.join("unsupported.rmeta")),
    );
    success(&rustc_accepts);
    let rejected = execute(
        Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
            .current_dir(&fixture)
            .env("LUSTC", &lust)
            .env("LUST_NO_DAEMON", "1")
            .args(["--engine", "lust", "check"])
            .arg(fixture.join("unsupported.rs")),
    );
    assert!(!rejected.status.success(), "Lust restriction was hidden by another compiler");
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("L0405"),
        "expected Lust async rejection: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
}

#[test]
#[ignore = "requires matching nightly Cargo and LUST_CRANELIFT_BACKEND built with unwinding"]
fn actual_cranelift_keeps_full_rust_output_including_unwinding() {
    let backend = std::env::var_os("LUST_CRANELIFT_BACKEND")
        .expect("set LUST_CRANELIFT_BACKEND to the matching custom backend built with --features unwinding");
    assert!(
        Path::new(&backend).is_absolute() && Path::new(&backend).is_file(),
        "LUST_CRANELIFT_BACKEND must name an existing absolute backend library: {backend:?}",
    );
    let scratch = Scratch::new("actual-cranelift");
    let fixture = scratch.fixture("full-rust");

    // Build separately, then run the final Cargo binary directly. A runtime
    // panic must not be reported as an unexplained `cargo run` compilation error.
    let baseline_build = execute(command(&scratch, &fixture, "llvm", None).args(["build", "--offline", "--locked", "--workspace"]));
    success_at("baseline Cargo compilation", &baseline_build);
    let candidate_build =
        execute(command(&scratch, &fixture, "cranelift", Some("cranelift")).args(["build", "--offline", "--locked", "--workspace"]));
    success_at("Cranelift Cargo compilation, before any program execution", &candidate_build);

    let executable_name = format!("fixture-app{}", std::env::consts::EXE_SUFFIX);
    let baseline = execute(Command::new(scratch.0.join("llvm/debug").join(&executable_name)).current_dir(&fixture));
    success_at("baseline binary execution, after successful compilation", &baseline);
    assert_eq!(baseline.stdout, EXPECTED.as_bytes());
    assert_eq!(baseline.stderr, EXPECTED_PANIC_TRACE.as_bytes());

    let cranelift = execute(Command::new(scratch.0.join("cranelift/debug").join(&executable_name)).current_dir(&fixture));
    success_at("Cranelift binary execution, after successful compilation; panic trace follows", &cranelift);
    assert_eq!(cranelift.status.code(), baseline.status.code());
    assert_eq!(cranelift.stdout, baseline.stdout);
    assert_eq!(cranelift.stderr, baseline.stderr);
    let tested =
        execute(command(&scratch, &fixture, "cranelift", Some("cranelift")).args(["test", "--offline", "--locked", "--workspace"]));
    success_at("Cranelift workspace unit tests and doctests", &tested);
}
