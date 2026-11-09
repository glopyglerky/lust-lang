use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn words(input: &[&str]) -> Vec<OsString> {
    input.iter().map(OsString::from).collect()
}

fn request(input: &[&str]) -> cargo_lust::Request {
    let cargo_lust::Action::Execute(request) = cargo_lust::parse(&words(input)).unwrap() else { panic!() };
    request
}

fn programs() -> cargo_lust::Programs {
    cargo_lust::Programs { cargo: "unused cargo".into(), lust: "unused lust".into(), explicit_cargo: false, cranelift_backend: None }
}

#[test]
fn frontend_plan_preserves_cargo_and_compiler_argument_boundaries() {
    let input = request(&[
        "--bluefe-root",
        "/not installed toolchain",
        "--frontend-jobs",
        "4",
        "rustc",
        "-p",
        "package with spaces",
        "--lib",
        "--",
        "-Cdebuginfo=2",
        "--cfg",
        "project_setting=\"kept\"",
    ]);
    let selected = cargo_lust::plan(&input, programs()).unwrap();
    assert_eq!(
        selected.args,
        words(&[
            "rustc",
            "-p",
            "package with spaces",
            "--lib",
            "--",
            "-Cdebuginfo=2",
            "--cfg",
            "project_setting=\"kept\"",
            "-Zunstable-options",
            "--jobs-frontend=4",
        ])
    );
    assert!(selected.environment_remove.is_empty());
    assert_eq!(selected.environment.len(), 4);
    assert!(selected.experimental);
}

#[test]
fn frontend_option_requires_explicit_toolchain_and_target_command() {
    for input in [
        vec!["--bluefe-root", "/abs", "--frontend-jobs", "0", "rustc"],
        vec!["--bluefe-root", "/abs", "--frontend-jobs=-1", "rustc"],
        vec!["--bluefe-root", "/abs", "--frontend-jobs"],
        vec!["--frontend-jobs", "4", "rustc"],
        vec!["--bluefe-root", "/abs", "--frontend-jobs=4", "check"],
        vec!["--bluefe-root", "/abs", "--frontend-jobs=4", "--engine", "cranelift", "rustc"],
        vec!["--bluefe-root", "/abs", "--frontend-jobs=4", "--frontend-jobs=2", "rustc"],
    ] {
        assert!(cargo_lust::parse(&words(&input)).is_err(), "{input:?}");
    }
    for flags in [vec!["--jobs-frontend=2"], vec!["--jobs-frontend", "2"], vec!["-Zthreads=2"], vec!["-Z", "threads=2"]] {
        let mut input = vec!["--bluefe-root", "/abs", "--frontend-jobs=4", "rustc", "--"];
        input.extend(flags);
        assert!(cargo_lust::parse(&words(&input)).is_err(), "{input:?}");
    }
    let mut forged = request(&["--bluefe-root", "/abs", "rustc"]);
    forged.frontend_jobs = Some(0);
    assert!(cargo_lust::plan(&forged, programs()).is_err());
}

#[test]
fn frontend_plan_is_pure_and_inserts_only_one_argument_boundary() {
    let selected = cargo_lust::plan(
        &request(&["--bluefe-root", "/not installed toolchain", "--frontend-jobs=1", "rustc", "--profile", "check"]),
        programs(),
    )
    .unwrap();
    assert_eq!(selected.args, words(&["rustc", "--profile", "check", "--", "-Zunstable-options", "--jobs-frontend=1"]));
    let output = Command::new(env!("CARGO_BIN_EXE_lustr"))
        .args(["--bluefe-root", "/not installed toolchain", "--frontend-jobs=4", "plan", "rustc", "--lib"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"{"));
    assert!(output.stderr.is_empty());
    let regular = request(&["rustc", "--", "--frontend-jobs=4"]);
    assert_eq!(regular.frontend_jobs, None);
    assert_eq!(cargo_lust::plan(&regular, programs()).unwrap().args, regular.args);
}

#[cfg(unix)]
#[test]
fn frontend_preserves_every_nonzero_argument_byte() {
    use std::os::unix::ffi::OsStringExt;
    for byte in 1..=255 {
        let argument = OsString::from_vec(vec![b'x', byte, b'y']);
        let mut input = words(&["--bluefe-root", "/abs", "--frontend-jobs=4", "rustc", "--"]);
        input.push(argument.clone());
        let cargo_lust::Action::Execute(request) = cargo_lust::parse(&input).unwrap() else { panic!() };
        let plan = cargo_lust::plan(&request, programs()).unwrap();
        assert_eq!(plan.args[2], argument);
        assert_eq!(plan.args.len(), 5);
    }
}

fn copy(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "target" {
            continue;
        }
        assert!(!entry.file_type().unwrap().is_symlink());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

#[test]
#[ignore = "requires LUST_TEST_BLUEFE_ROOT with the produced Linux toolchain"]
fn real_frontend_modes_preserve_settings_wrappers_runtime_tests_and_rejection() {
    let root = PathBuf::from(std::env::var_os("LUST_TEST_BLUEFE_ROOT").expect("explicit produced BlueFe required"));
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temporary = std::env::temp_dir().join(format!("frontend qualification {} {nonce}", std::process::id()));
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(temporary.clone());
    let fixture = temporary.join("full rust");
    copy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/full-rust"), &fixture);
    fs::create_dir(fixture.join(".cargo")).unwrap();
    fs::write(
        fixture.join(".cargo/config.toml"),
        "[build]\nrustflags = ['--cfg', 'from_config', '--check-cfg', 'cfg(from_config,from_environment)']\n",
    )
    .unwrap();
    let app = fixture.join("app/src/main.rs");
    let original = fs::read_to_string(&app).unwrap();
    fs::write(&app, format!("#[cfg(not(any(from_config,from_environment)))] compile_error!(\"caller flags disappeared\");\n{original}"))
        .unwrap();
    let wrapper_source = temporary.join("wrapper.rs");
    fs::write(
        &wrapper_source,
        r#"
use std::io::Write;
fn main() {
    let mut args=std::env::args_os().skip(1);
    let program=args.next().unwrap(); let args:Vec<_>=args.collect();
    let mut log=std::fs::OpenOptions::new().create(true).append(true).open(std::env::var_os("FRONTEND_WRAPPER_LOG").unwrap()).unwrap();
    writeln!(log,"{program:?} {args:?}").unwrap();
    let status=std::process::Command::new(program).args(args).status().unwrap();
    std::process::exit(status.code().unwrap_or(1));
}
"#,
    )
    .unwrap();
    let wrapper = temporary.join("compiler wrapper");
    let built = Command::new(root.join("bin/rustc")).arg(&wrapper_source).arg("-o").arg(&wrapper).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let log = temporary.join("wrappers.log");
    let run = |jobs: &str, args: &[&str], environment_flags: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lustr"));
        command
            .arg("--bluefe-root")
            .arg(&root)
            .arg("--frontend-jobs")
            .arg(jobs)
            .args(args)
            .current_dir(&fixture)
            .env("CARGO_TARGET_DIR", temporary.join(format!("target-{jobs}")))
            .env("RUSTC_WRAPPER", &wrapper)
            .env("RUSTC_WORKSPACE_WRAPPER", &wrapper)
            .env("FRONTEND_WRAPPER_LOG", &log)
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS");
        if environment_flags {
            command.env(
                "CARGO_ENCODED_RUSTFLAGS",
                ["--cfg", "from_environment", "--check-cfg", "cfg(from_config,from_environment)", "-Cdebug-assertions=yes"].join("\x1f"),
            );
        }
        command.output().unwrap()
    };
    for jobs in ["1", "4"] {
        let checked =
            run(jobs, &["rustc", "--profile", "check", "-p", "fixture-app", "--bin", "fixture-app", "--offline", "--locked", "-vv"], false);
        assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));
        let stderr = String::from_utf8_lossy(&checked.stderr);
        let root_command = stderr.lines().find(|line| line.contains("Running") && line.contains("--crate-name fixture_app ")).unwrap();
        assert!(root_command.contains(&format!("--jobs-frontend={jobs}")));
        assert!(root_command.contains("--emit=dep-info,metadata"));
        assert!(root_command.contains("--cfg from_config"));
        let built = run(jobs, &["rustc", "-p", "fixture-app", "--bin", "fixture-app", "--offline", "--locked"], true);
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        // The wrapper changes Cargo's artifact hash; locate the generated executable without assuming its private directory layout.
        let mut candidates = Vec::new();
        fn executables(path: &Path, prefix: &str, candidates: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    executables(&entry.path(), prefix, candidates);
                } else if entry.file_name().to_string_lossy().starts_with(prefix) && entry.path().extension().is_none() {
                    if fs::read(entry.path()).unwrap().starts_with(b"\x7fELF") {
                        candidates.push(entry.path());
                    }
                }
            }
        }
        executables(&temporary.join(format!("target-{jobs}")), "fixture-app", &mut candidates);
        // Cargo's bin alias and hashed executable can both name the same output.
        assert!(!candidates.is_empty());
        let output = Command::new(&candidates[0]).output().unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("panic=1 drops=1"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("catch_unwind returned; caught=true drops=1"));
    }
    let tested = run("4", &["rustc", "--profile", "test", "-p", "fixture-core", "--lib", "--offline", "--locked", "-vv"], true);
    assert!(tested.status.success(), "{}", String::from_utf8_lossy(&tested.stderr));
    assert!(
        String::from_utf8_lossy(&tested.stderr)
            .lines()
            .any(|line| line.contains("--crate-name fixture_core ") && line.contains("--test") && line.contains("--jobs-frontend=4"))
    );
    let mut tests = Vec::new();
    fn find_tests(path: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                find_tests(&entry.path(), found);
            } else if entry.file_name().to_string_lossy().starts_with("fixture_core-")
                && entry.path().extension().is_none()
                && fs::read(entry.path()).unwrap().starts_with(b"\x7fELF")
            {
                found.push(entry.path());
            }
        }
    }
    find_tests(&temporary.join("target-4"), &mut tests);
    assert_eq!(tests.len(), 1);
    let executed = Command::new(&tests[0]).output().unwrap();
    assert!(executed.status.success());
    assert!(String::from_utf8_lossy(&executed.stdout).contains("features_keep_their_rust_semantics"));
    assert!(fs::read_to_string(&log).unwrap().contains("--jobs-frontend=4"));
    fs::write(&app, "fn main() { let mut v=vec![1]; let p=&v[0]; v.push(2); println!(\"{p}\"); }\n").unwrap();
    let rejected = run("4", &["rustc", "--profile", "check", "-p", "fixture-app", "--bin", "fixture-app", "--offline", "--locked"], true);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E0502"));
}
