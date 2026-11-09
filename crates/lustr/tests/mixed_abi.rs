//! Real crate-boundary qualification. A fresh compilation and verified compiler
//! command are prerequisites for claiming either backend participated.

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_STDOUT: &[u8] = b"mixed-abi passed: aggregates=3 generics=2 traits=2 niches=6 enums=3 allocation-directions=2 unwind-directions=2 atomics=8000 tls-threads=2 c-callbacks=1\n";
const EXPECTED_STDERR: &[u8] = b"mixed-abi panic hook: consumer-origin\nmixed-abi panic hook: provider-origin\n";
// rustc shipped by the nightly-2026-11-05 backend pin. Updating this gate
// requires qualifying the new pair; a toolchain name alone is not identity.
const PINNED_RUSTC_COMMIT: &str = "2d20b42b883dcfc014f3c2c56081eeb6091c2179";
const PINNED_HOST: &str = "x86_64-unknown-linux-gnu";
const CRATES: [&str; 3] = ["mixed_abi_contract", "mixed_abi_provider", "mixed_abi_app"];
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch {
    root: PathBuf,
    fixture: PathBuf,
    evidence: Option<PathBuf>,
}

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let name = format!("cargo-lust-mixed-abi-{label}-{}-{nonce}-{serial}", std::process::id());
        let root = std::env::temp_dir().join(&name);
        fs::create_dir(&root).unwrap();
        let fixture = root.join("fixture with spaces");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/mixed-abi");
        copy_tree(&source, &fixture);
        let evidence = std::env::var_os("LUST_TEST_ABI_EVIDENCE").map(|base| {
            let base = PathBuf::from(base);
            assert!(base.is_absolute(), "LUST_TEST_ABI_EVIDENCE must be an absolute directory");
            fs::create_dir_all(&base).unwrap();
            let destination = base.join(name);
            fs::create_dir(&destination).unwrap();
            destination
        });
        Self { root, fixture, evidence }
    }

    fn command(&self, target: &str, engine: Option<&str>) -> Command {
        let mut command = match engine {
            None => Command::new(cargo()),
            Some(engine) => {
                let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-lust"));
                command.env("LUST_CARGO", cargo()).args(["--engine", engine]);
                command
            }
        };
        command.current_dir(&self.fixture).env("CARGO_TARGET_DIR", self.root.join(target));
        // Prevent the test runner's compilation settings from becoming the
        // fixture baseline. Every intended backend is in this Cargo command.
        for key in [
            "CARGO_PROFILE_DEV_CODEGEN_BACKEND",
            "CARGO_PROFILE_TEST_CODEGEN_BACKEND",
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_BUILD_RUSTFLAGS",
            "RUSTC_WRAPPER",
            "RUSTC_WORKSPACE_WRAPPER",
        ] {
            command.env_remove(key);
        }
        command
    }

    fn execute(&self, stage: &str, command: &mut Command) -> Output {
        let output = command.output().unwrap_or_else(|error| panic!("could not execute {stage}: {command:?}: {error}"));
        if let Some(evidence) = &self.evidence {
            fs::write(evidence.join(format!("{stage}.stdout")), &output.stdout).unwrap();
            fs::write(evidence.join(format!("{stage}.stderr")), &output.stderr).unwrap();
            fs::write(evidence.join(format!("{stage}.status")), format!("{}\n", output.status)).unwrap();
        }
        assert!(
            output.status.success(),
            "{stage} failed: status={}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn run_binary(&self, target: &str) -> Output {
        let binary = self.root.join(target).join("debug").join(format!("mixed-abi-app{}", std::env::consts::EXE_SUFFIX));
        let output = self.execute(&format!("{target}-run"), Command::new(binary).current_dir(&self.fixture));
        assert_eq!(output.stdout, EXPECTED_STDOUT, "fixture assertions did not finish for {target}");
        assert_eq!(output.stderr, EXPECTED_STDERR, "both panic origins must be observed for {target}");
        output
    }

    fn completed(&self, summary: &str) {
        if let Some(evidence) = &self.evidence {
            fs::write(evidence.join("passed.txt"), summary).unwrap();
            eprintln!("Mixed ABI raw command and runtime evidence: {}", evidence.display());
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        // A documented direct Cargo run creates this build directory in the
        // fixture. Qualification always builds into its own empty target dirs.
        if entry.file_name() == OsStr::new("target") {
            continue;
        }
        let target = destination.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink(), "fixture must not contain symlinks");
        if kind.is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn cargo() -> OsString {
    std::env::var_os("LUST_TEST_CARGO").unwrap_or_else(|| OsString::from(env!("CARGO")))
}

fn build_arguments<'a>(command: &'a mut Command, operation: &str) -> &'a mut Command {
    command.args([operation, "--offline", "--locked", "--workspace", "-vv", "--color", "never"]);
    if operation == "test" {
        command.arg("--all-targets");
    }
    command
}

#[derive(Clone, Copy)]
enum Backend<'a> {
    DefaultLlvm,
    Explicit(&'a str),
}

/// Tokenize Cargo's displayed argv; never execute it as shell input. Newlines
/// inside quoted environment values belong to the same compiler invocation.
fn command_words(command: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = command.chars().peekable();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (Some('\''), _) => word.push(ch),
            (Some('"'), '\\') => {
                let next = chars.next().ok_or("incomplete quoted escape in Cargo command")?;
                if !matches!(next, '$' | '`' | '"' | '\\' | '\n') {
                    word.push('\\');
                }
                if next != '\n' {
                    word.push(next);
                }
            }
            (None, '\\') => {
                let next = chars.next().ok_or("incomplete escape in Cargo command")?;
                if next != '\n' {
                    word.push(next);
                    started = true;
                }
            }
            (Some('"'), _) => word.push(ch),
            (None, '\'' | '"') => {
                quote = Some(ch);
                started = true;
            }
            (None, ch) if ch.is_whitespace() => {
                if started {
                    result.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => {
                word.push(ch);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err("unterminated quote in Cargo command".into());
    }
    if started {
        result.push(word);
    }
    Ok(result)
}

fn verbose_commands(log: &str) -> Result<Vec<Vec<String>>, String> {
    let mut commands = Vec::new();
    let mut remaining = log;
    while let Some(start) = remaining
        .lines()
        .scan(0, |offset, line| {
            let position = *offset;
            *offset += line.len() + 1;
            Some((position, line))
        })
        .find_map(|(position, line)| {
            let trimmed = line.trim_start();
            trimmed.strip_prefix("Running `").map(|_| position + line.len() - trimmed.len() + "Running `".len())
        })
    {
        remaining = &remaining[start..];
        let mut quote = None;
        let mut escaped = false;
        let mut end = None;
        for (index, ch) in remaining.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            match (quote, ch) {
                (Some('\''), '\'') | (Some('"'), '"') => quote = None,
                (Some('\''), _) => {}
                (_, '\\') => escaped = true,
                (None, '\'' | '"') => quote = Some(ch),
                (None, '`') => {
                    if remaining[index + 1..].starts_with('\n')
                        || remaining[index + 1..].starts_with("\r\n")
                        || index + 1 == remaining.len()
                    {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.ok_or("unterminated Cargo Running command")?;
        commands.push(command_words(&remaining[..end])?);
        remaining = &remaining[end + 1..];
    }
    Ok(commands)
}

fn verify_backends(stderr: &[u8], expected: &[(&str, Backend<'_>)]) -> Result<BTreeSet<String>, String> {
    let log = std::str::from_utf8(stderr).map_err(|error| format!("Cargo diagnostics are not UTF-8: {error}"))?;
    let commands = verbose_commands(log)?;
    let mut compilers = BTreeSet::new();
    for (crate_name, expected_backend) in expected {
        let mut matched = 0;
        for words in &commands {
            let Some(index) = words.windows(2).position(|pair| pair[0] == "--crate-name" && pair[1] == *crate_name) else { continue };
            // rustdoc also has --crate-name; require rustc artifact emission.
            if !words.iter().any(|word| word.starts_with("--emit=")) {
                continue;
            }
            matched += 1;
            let backends: Vec<_> = words
                .iter()
                .enumerate()
                .filter_map(|(index, word)| {
                    word.strip_prefix("-Zcodegen-backend=")
                        .or_else(|| if word == "-Z" { words.get(index + 1)?.strip_prefix("codegen-backend=") } else { None })
                })
                .collect();
            let valid = match expected_backend {
                Backend::DefaultLlvm => backends.is_empty() || backends == ["llvm"],
                Backend::Explicit(backend) => backends == [*backend],
            };
            if !valid {
                return Err(format!("unexpected backend for {crate_name}: {backends:?}; actual command: {words:?}"));
            }
            let compiler =
                index.checked_sub(1).and_then(|index| words.get(index)).ok_or("compiler executable missing before --crate-name")?;
            compilers.insert(compiler.clone());
        }
        if matched == 0 {
            return Err(format!("no fresh rustc compilation for {crate_name}; a cached artifact does not qualify; Cargo log:\n{log}"));
        }
    }
    Ok(compilers)
}

fn verified(stderr: &[u8], expected: &[(&str, Backend<'_>)]) -> BTreeSet<String> {
    verify_backends(stderr, expected).unwrap_or_else(|error| panic!("{error}"))
}

fn assert_test_ran(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test tests::bidirectional_abi_contract ... ok"), "the contract test did not run: {stdout}");
}

#[test]
fn ordinary_cargo_and_coordinator_preserve_the_bidirectional_contract() {
    let mut scratch = Scratch::new("ordinary");
    // Follow the README's direct-run workflow first. Its normal Cargo output
    // must not prevent a subsequent fresh qualification of the same sources.
    let mut documented = scratch.command("documented", None);
    documented.args(["run", "--offline", "--locked", "--quiet"]);
    documented.env("CARGO_TARGET_DIR", scratch.fixture.join("target"));
    let ran = scratch.execute("documented-direct-run", &mut documented);
    assert_eq!(ran.stdout, EXPECTED_STDOUT);
    assert_eq!(ran.stderr, EXPECTED_STDERR);
    assert!(scratch.fixture.join("target").is_dir());
    let fresh = scratch.root.join("qualified sources with spaces");
    copy_tree(&scratch.fixture, &fresh);
    assert!(!fresh.join("target").exists(), "generated artifacts must not enter the qualification copy");
    scratch.fixture = fresh;
    let expected: Vec<_> = CRATES.iter().map(|name| (*name, Backend::DefaultLlvm)).collect();
    let baseline = scratch.execute("llvm-build", build_arguments(&mut scratch.command("llvm", None), "build"));
    let baseline_compilers = verified(&baseline.stderr, &expected);
    let baseline = scratch.run_binary("llvm");
    let tested = scratch.execute("llvm-test", build_arguments(&mut scratch.command("llvm", None), "test"));
    assert_eq!(verified(&tested.stderr, &expected), baseline_compilers);
    assert_test_ran(&tested);

    let built = scratch.execute("coordinator-build", build_arguments(&mut scratch.command("coordinator", Some("rustc")), "build"));
    assert_eq!(verified(&built.stderr, &expected), baseline_compilers);
    let candidate = scratch.run_binary("coordinator");
    assert_eq!(candidate.status.code(), baseline.status.code());
    assert_eq!(candidate.stdout, baseline.stdout);
    assert_eq!(candidate.stderr, baseline.stderr);
    let tested = scratch.execute("coordinator-test", build_arguments(&mut scratch.command("coordinator", Some("rustc")), "test"));
    assert_eq!(verified(&tested.stderr, &expected), baseline_compilers);
    assert_test_ran(&tested);
    scratch.completed("passed: fresh ordinary Cargo and coordinated rustc builds, direct executables and Cargo test harnesses\n");
}

fn mixed_command(scratch: &Scratch, target: &str, operation: &str, llvm_packages: &[&str]) -> Command {
    let mut command = scratch.command(target, Some("cranelift"));
    build_arguments(&mut command, operation);
    for profile in ["dev", "test"] {
        for package in llvm_packages {
            command.arg("--config").arg(format!("profile.{profile}.package.{package}.codegen-backend=\"llvm\""));
        }
    }
    command
}

#[test]
#[ignore = "requires pinned Linux x86_64 nightly-2026-11-05 Cargo and LUST_CRANELIFT_BACKEND built with unwinding"]
fn actual_cranelift_preserves_both_directions_of_the_rust_abi() {
    let backend = std::env::var_os("LUST_CRANELIFT_BACKEND").expect("set LUST_CRANELIFT_BACKEND to the pinned unwind-capable backend");
    let backend_path = Path::new(&backend);
    assert!(backend_path.is_absolute() && backend_path.is_file(), "LUST_CRANELIFT_BACKEND must be an existing absolute library path");
    let backend = backend.to_str().expect("verbose qualification requires a UTF-8 backend path");
    let scratch = Scratch::new("actual-cranelift");
    let baseline_expected: Vec<_> = CRATES.iter().map(|name| (*name, Backend::DefaultLlvm)).collect();
    let built = scratch.execute("llvm-build", build_arguments(&mut scratch.command("llvm", None), "build"));
    let baseline_compilers = verified(&built.stderr, &baseline_expected);
    assert_eq!(baseline_compilers.len(), 1, "all crate boundaries must use one matching rustc frontend");
    let compiler = baseline_compilers.first().unwrap();
    let identity = scratch.execute("rustc-identity", Command::new(compiler).arg("-vV"));
    let identity = String::from_utf8(identity.stdout).unwrap();
    let expected_commit = std::env::var("LUST_TEST_RUSTC_COMMIT").unwrap_or_else(|_| PINNED_RUSTC_COMMIT.to_owned());
    assert!(
        expected_commit.len() == 40 && expected_commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "qualification requires a full compiler commit"
    );
    assert!(
        identity.lines().any(|line| line == format!("commit-hash: {expected_commit}")),
        "backend ABI gate requires the pinned rustc commit; observed:\n{identity}"
    );
    assert!(
        identity.lines().any(|line| line == format!("host: {PINNED_HOST}")),
        "this backend pair is qualified only for {PINNED_HOST}; observed:\n{identity}"
    );
    let baseline = scratch.run_binary("llvm");
    let tested = scratch.execute("llvm-test", build_arguments(&mut scratch.command("llvm", None), "test"));
    assert_eq!(verified(&tested.stderr, &baseline_expected), baseline_compilers);
    assert_test_ran(&tested);

    for (target, llvm_packages, expected) in [
        (
            "cranelift-consumer",
            ["mixed-abi-contract", "mixed-abi-provider"],
            [
                ("mixed_abi_contract", Backend::Explicit("llvm")),
                ("mixed_abi_provider", Backend::Explicit("llvm")),
                ("mixed_abi_app", Backend::Explicit(backend)),
            ],
        ),
        (
            "cranelift-provider",
            ["mixed-abi-contract", "mixed-abi-app"],
            [
                ("mixed_abi_contract", Backend::Explicit("llvm")),
                ("mixed_abi_provider", Backend::Explicit(backend)),
                ("mixed_abi_app", Backend::Explicit("llvm")),
            ],
        ),
    ] {
        let built = scratch.execute(&format!("{target}-build"), &mut mixed_command(&scratch, target, "build", &llvm_packages));
        assert_eq!(verified(&built.stderr, &expected), baseline_compilers, "mixed crates used a different frontend");
        let candidate = scratch.run_binary(target);
        assert_eq!(candidate.status.code(), baseline.status.code());
        assert_eq!(candidate.stdout, baseline.stdout);
        assert_eq!(candidate.stderr, baseline.stderr);
        let tested = scratch.execute(&format!("{target}-test"), &mut mixed_command(&scratch, target, "test", &llvm_packages));
        assert_eq!(verified(&tested.stderr, &expected), baseline_compilers, "test harnesses used a different frontend");
        assert_test_ran(&tested);
    }
    scratch.completed(&format!("passed: fresh all-LLVM baseline and both mixed LLVM/Cranelift selections; direct execution and Cargo tests; complete destructor/panic contract\nrustc_commit={expected_commit}\nhost={PINNED_HOST}\nbackend={backend}\n"));
}

#[test]
fn compiler_evidence_requires_fresh_commands_and_exact_backend_selection() {
    let expected = [("mixed_abi_app", Backend::Explicit("/backend with spaces/clif.so"))];
    let valid = b"     Running `CARGO_PKG_DESCRIPTION='two lines\nwith a ` backtick' '/rustc path/rustc' --crate-name mixed_abi_app app/src/main.rs --emit=dep-info,link -Z 'codegen-backend=/backend with spaces/clif.so'`\n";
    assert_eq!(verified(valid, &expected), BTreeSet::from([String::from("/rustc path/rustc")]));
    assert!(verify_backends(b"       Fresh mixed-abi-app v0.0.0\n", &expected).is_err());
    assert!(verify_backends(b"     Running `rustc --crate-name mixed_abi_app app/src/main.rs --emit=dep-info,link`\n", &expected).is_err());
    assert!(
        verify_backends(
            b"     Running `rustc --crate-name mixed_abi_app app/src/main.rs --emit=dep-info,link -Zcodegen-backend=/old/clif.so`\n",
            &expected
        )
        .is_err()
    );
    assert!(
        verify_backends(b"     Running `rustdoc --crate-name mixed_abi_app app/src/main.rs -Zcodegen-backend=/old/clif.so`\n", &expected)
            .is_err()
    );
    assert!(verify_backends(b"     Running `rustc --crate-name mixed_abi_app --emit=link -Z 'codegen-backend=/backend with spaces/clif.so' -Zcodegen-backend=llvm`\n", &expected).is_err());
}
