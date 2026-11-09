use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn explicit_root_plan_is_exact_manual_tool_selection_without_probes() {
    let root = std::env::temp_dir().join("not installed BlueFe toolchain");
    let args: Vec<_> = [
        OsString::from("--bluefe-root"),
        root.as_os_str().into(),
        OsString::from("plan"),
        OsString::from("build"),
        OsString::from("--"),
        OsString::from("argument with spaces"),
    ]
    .into();
    let cargo_lust::Action::Execute(request) = cargo_lust::parse(&args).unwrap() else { panic!() };
    let selected = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
    assert_eq!(selected.program, root.join("bin").join(format!("cargo{}", std::env::consts::EXE_SUFFIX)));
    assert_eq!(selected.args, request.args);
    assert_eq!(selected.environment.len(), 4);
    for (variable, name) in [("CARGO", "cargo"), ("LUST_CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")] {
        let path = root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        assert!(selected.environment.contains(&(variable.into(), path.into_os_string())));
    }
    let cli = Command::new(env!("CARGO_BIN_EXE_lustr")).args(args).output().unwrap();
    assert!(cli.status.success());
    assert!(cli.stdout.starts_with(b"{"));
    assert!(cli.stderr.is_empty());
}

#[test]
fn ambiguous_root_selections_and_missing_tools_fail() {
    for args in [
        vec!["--bluefe-root"],
        vec!["--bluefe-root", "relative", "build"],
        vec!["--bluefe-root", "/abs", "+stable", "build"],
        vec!["--bluefe-root", "/abs", "--engine", "lust", "build"],
        vec!["--bluefe-root", "/abs", "--bluefe-root", "/abs", "build"],
    ] {
        assert!(cargo_lust::parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_err());
    }
    let root = std::env::temp_dir().join(format!("absent-bluefe-{}", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_lustr")).arg("--bluefe-root").arg(root).arg("build").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing"));
}

#[cfg(unix)]
#[test]
fn root_plan_preserves_every_nonzero_argument_byte() {
    use std::os::unix::ffi::OsStringExt;
    for byte in 1..=255 {
        let argument = OsString::from_vec(vec![b'x', byte, b'y']);
        let input = vec!["--bluefe-root".into(), "/nonexistent".into(), "build".into(), argument.clone()];
        let cargo_lust::Action::Execute(request) = cargo_lust::parse(&input).unwrap() else { panic!() };
        let selected = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
        assert_eq!(selected.args[1], argument);
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
#[ignore = "requires LUST_TEST_BLUEFE_ROOT with a complete native/wasm produced sysroot"]
fn real_bluefe_builds_tests_rejects_and_cross_compiles() {
    let root = PathBuf::from(std::env::var_os("LUST_TEST_BLUEFE_ROOT").expect("explicit produced BlueFe sysroot required"));
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temporary = std::env::temp_dir().join(format!("bluefe qualification {} {nonce}", std::process::id()));
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(temporary.clone());
    let fixture = temporary.join("full-rust");
    copy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/full-rust"), &fixture);
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_lustr"))
            .arg("--bluefe-root")
            .arg(&root)
            .args(args)
            .current_dir(&fixture)
            .env("CARGO_TARGET_DIR", temporary.join("target"))
            .output()
            .unwrap()
    };
    for args in [
        vec!["run", "--offline", "--locked", "--target", "x86_64-unknown-linux-gnu"],
        vec!["test", "--workspace", "--offline", "--locked", "--target", "x86_64-unknown-linux-gnu"],
        vec!["build", "--workspace", "--offline", "--locked", "--target", "wasm32-unknown-unknown"],
    ] {
        let result = run(&args);
        assert!(result.status.success(), "{args:?}: {}", String::from_utf8_lossy(&result.stderr));
        if args[0] == "run" {
            assert!(String::from_utf8_lossy(&result.stdout).contains("panic=1 drops=1"));
        }
        if args[0] == "test" {
            assert!(String::from_utf8_lossy(&result.stdout).contains("body_probe"));
        }
    }
    let wasm = fs::read(temporary.join("target/wasm32-unknown-unknown/debug/fixture-app.wasm")).unwrap();
    assert_eq!(&wasm[..8], b"\0asm\x01\0\0\0");
    fs::write(fixture.join("app/src/main.rs"), "fn main() { let mut v=vec![1]; let p=&v[0]; v.push(2); println!(\"{p}\"); }\n").unwrap();
    let rejected = run(&["check", "--offline", "--locked"]);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("E0502"));
    let doctor = run(&["doctor"]);
    assert!(doctor.status.success());
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("1.102.0"));
}
