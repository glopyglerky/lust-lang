//! Explicit opt-in qualification of coordinator-routed tests with matched tools.
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn selected_root_and_actual_backend_are_observed() {
    let Some(root) = std::env::var_os("LUSTR_BENCH_TEST_BLUEFE_ROOT").map(PathBuf::from) else {
        eprintln!("not run: explicit BlueFe root required");
        return;
    };
    let backend = PathBuf::from(std::env::var_os("LUSTR_BENCH_TEST_CRANELIFT_BACKEND").expect("explicit matching backend required"));
    assert!(backend.is_absolute() && backend.is_file());
    let out = PathBuf::from(std::env::var_os("LUSTR_BENCH_TEST_OUTPUT").expect("explicit output required")).with_extension("routed");
    fs::create_dir(&out).unwrap();
    let source = out.join("snapshot");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("src")).unwrap();
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/tiny");
    for path in ["Cargo.toml", "Cargo.lock", "src/main.rs", "reject.rs"] {
        fs::copy(original.join(path), source.join(path)).unwrap();
    }
    for path in ["home", "cargo-home", "tmp"] {
        fs::create_dir(out.join(path)).unwrap();
    }
    let selected_command = |engine: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lustr-bench-gate"));
        command
            .env_clear()
            .current_dir(&source)
            .env("PATH", format!("{}:/usr/bin:/bin", root.join("bin").display()))
            .env("HOME", out.join("home"))
            .env("TMPDIR", out.join("tmp"))
            .env("CARGO_HOME", out.join("cargo-home"))
            .env("CARGO", root.join("bin/cargo"))
            .env("RUSTC", root.join("bin/rustc"))
            .env("RUSTDOC", root.join("bin/rustdoc"))
            .env("LUST_CRANELIFT_BACKEND", &backend)
            .env("CARGO_TARGET_DIR", out.join("targets").join(engine))
            .env("CARGO_BUILD_JOBS", "2")
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TERM_COLOR", "never")
            .args([
                "tests-via".as_ref(),
                root.join("bin/lustr").as_os_str(),
                "--bluefe-root".as_ref(),
                root.as_os_str(),
                "--engine".as_ref(),
                engine.as_ref(),
                "test".as_ref(),
                "--locked".as_ref(),
                "--offline".as_ref(),
                "-vv".as_ref(),
            ]);
        command
    };
    for engine in ["auto", "cranelift"] {
        let output = selected_command(engine).output().unwrap();
        fs::write(out.join(format!("{engine}.stdout")), &output.stdout).unwrap();
        fs::write(out.join(format!("{engine}.stderr")), &output.stderr).unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let summary: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(summary["kind"], "tests");
        assert!(summary["passed_test_names"].as_array().unwrap().iter().any(|name| name.as_str().unwrap().contains("runtime_and_cleanup")));
        let stderr = String::from_utf8_lossy(&output.stderr);
        let actual = stderr
            .lines()
            .find(|line| {
                line.contains("Running")
                    && line.contains("--crate-name bench_tiny ")
                    && line.contains(root.join("bin/rustc").to_str().unwrap())
                    && line.contains("--test")
            })
            .expect("actual root compiler command required");
        if engine == "cranelift" {
            assert!(actual.contains("codegen-backend") && actual.contains(backend.to_str().unwrap()));
        } else {
            assert!(!actual.contains("codegen-backend"));
        }
    }
    for variable in ["CARGO", "RUSTC", "RUSTDOC"] {
        let output = selected_command("auto").env(variable, out.join("wrong-tool")).output().unwrap();
        assert_eq!(output.status.code(), Some(42));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("routed {variable} selection must match --bluefe-root")));
        assert!(!stderr.contains("raw stdout:"));
        let missing = selected_command("auto").env_remove(variable).output().unwrap();
        assert_eq!(missing.status.code(), Some(42));
        assert!(missing.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&missing.stderr);
        assert!(stderr.contains("explicit tool selection is required"));
        assert!(!stderr.contains("raw stdout:"));
    }
    let output = selected_command("auto").arg("--no-run").output().unwrap();
    assert_eq!(output.status.code(), Some(42));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires an executing Cargo test command"));
}
