use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

struct Package {
    root: PathBuf,
    _installation: MutexGuard<'static, ()>,
}
static NEXT_PACKAGE: AtomicUsize = AtomicUsize::new(0);
static INSTALLATION: Mutex<()> = Mutex::new(());

impl Package {
    fn new() -> Self {
        let installation = INSTALLATION.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let sequence = NEXT_PACKAGE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("relocated lustr package {} {nonce} {sequence}", std::process::id()));
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::copy(env!("CARGO_BIN_EXE_lustr"), program(&root, "lustr")).unwrap();
        for name in ["cargo", "rustc", "rustdoc", "lust"] {
            fs::write(program(&root, name), []).unwrap();
        }
        fs::write(root.join("lustr-toolchain.json"), "{\"schema\":\"lustr.toolchain-package\",\"schema_version\":1}\n").unwrap();
        Self { root, _installation: installation }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(program(&self.root, "lustr"));
        for name in ["CARGO", "LUST_CARGO", "LUSTC", "RUSTC", "RUSTDOC", "RUSTUP_TOOLCHAIN", "LUST_CRANELIFT_BACKEND"] {
            command.env_remove(name);
        }
        command
    }
}

impl Drop for Package {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn program(root: &Path, name: &str) -> PathBuf {
    root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

fn json_path(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "\\\\").replace('"', "\\\"")
}

fn plan(output: Output) -> String {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn relocated_package_selects_its_cargo_and_sysroot_without_probes() {
    let package = Package::new();
    let output = plan(package.command().args(["plan", "build", "--offline", "--release"]).output().unwrap());
    assert!(output.contains(&format!("\"program\":\"{}\"", json_path(&program(&package.root, "cargo")))));
    for (key, name) in [("CARGO", "cargo"), ("LUST_CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")] {
        assert!(output.contains(&format!("\"{key}\":\"{}\"", json_path(&program(&package.root, name)))), "{output}");
    }
    assert!(output.contains("\"args\":[\"build\",\"--offline\",\"--release\"]"));
    assert!(output.contains("\"selected_engine\":\"rustc\""));
}

#[test]
fn inherited_package_cargo_keeps_the_matching_compiler() {
    let package = Package::new();
    let output = plan(package.command().env("CARGO", program(&package.root, "cargo")).args(["plan", "check"]).output().unwrap());
    assert!(output.contains(&format!("\"RUSTC\":\"{}\"", json_path(&program(&package.root, "rustc")))));
}

#[test]
fn explicit_tool_selection_takes_precedence_over_the_package() {
    let package = Package::new();
    for (key, value) in
        [("LUST_CARGO", "selected cargo"), ("CARGO", "selected cargo"), ("RUSTC", "selected rustc"), ("RUSTDOC", "selected rustdoc")]
    {
        let output = plan(package.command().env(key, value).args(["plan", "check"]).output().unwrap());
        assert!(output.contains("\"environment\":{}"), "{key}: {output}");
    }
    let output = plan(package.command().args(["+selected-toolchain", "plan", "check"]).output().unwrap());
    assert!(output.contains("\"program\":\"rustup\""));
    assert!(output.contains("\"environment\":{}"));
}

#[test]
fn environment_toolchain_uses_rustup_instead_of_the_bundled_cargo() {
    let package = Package::new();
    let output = plan(package.command().env("RUSTUP_TOOLCHAIN", "selected-toolchain").args(["plan", "check"]).output().unwrap());
    assert!(output.contains("\"program\":\"rustup\""));
    assert!(output.contains("\"args\":[\"run\",\"--\",\"selected-toolchain\",\"cargo\",\"check\"]"));
    assert!(output.contains("\"environment\":{}"));
}

#[test]
fn package_native_route_keeps_native_arguments_and_environment() {
    let package = Package::new();
    let output = plan(package.command().args(["--engine", "lust", "plan", "run", "source with spaces.rs"]).output().unwrap());
    assert!(output.contains(&format!("\"program\":\"{}\"", json_path(&program(&package.root, "lust")))));
    assert!(output.contains("\"selected_engine\":\"lust\""));
    assert!(output.contains("\"environment\":{}"));
    assert!(output.contains("\"args\":[\"run\",\"source with spaces.rs\"]"));
}

#[test]
fn ordinary_installation_keeps_existing_toolchain_selection() {
    let package = Package::new();
    fs::remove_file(package.root.join("lustr-toolchain.json")).unwrap();
    let output = plan(package.command().args(["plan", "check"]).output().unwrap());
    assert!(output.contains("\"program\":\"cargo\""));
    assert!(output.contains("\"environment\":{}"));
}

#[test]
fn incomplete_package_does_not_fall_back_to_an_unrelated_compiler() {
    let package = Package::new();
    fs::remove_file(program(&package.root, "rustc")).unwrap();
    let output = package.command().arg("build").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("toolchain is missing"));
}
