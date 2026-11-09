//! Explicitly opt-in: no implicit stock compiler acceptance.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn digest(p: &Path) -> String {
    Sha256::digest(fs::read(p).unwrap()).iter().map(|b| format!("{b:02x}")).collect()
}
fn cmd(argv: Value, exit: i32) -> Value {
    json!({"argv":argv,"expected_exit":exit})
}
#[test]
fn gate_identity_probe_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lustr-bench-gate")).arg("--version").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), concat!("lustr-bench-gate ", env!("CARGO_PKG_VERSION")));
}
#[test]
fn actual_bluefe_tiny() {
    let Some(root) = std::env::var_os("LUSTR_BENCH_TEST_BLUEFE_ROOT").map(PathBuf::from) else {
        eprintln!("not run: set LUSTR_BENCH_TEST_BLUEFE_ROOT and LUSTR_BENCH_TEST_OUTPUT for actual BlueFe integration");
        return;
    };
    let out = PathBuf::from(std::env::var_os("LUSTR_BENCH_TEST_OUTPUT").expect("explicit NAS generated-output directory"));
    fs::create_dir(&out).unwrap();
    let fixture = out.join("snapshot");
    fs::create_dir(&fixture).unwrap();
    fs::create_dir(fixture.join("src")).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/tiny");
    for file in ["Cargo.toml", "Cargo.lock", "src/main.rs", "reject.rs"] {
        fs::copy(input.join(file), fixture.join(file)).unwrap();
    }
    let mut tools: Vec<_> = ["cargo", "rustc", "rustdoc", "lustr"]
        .iter()
        .map(|n| {
            let p = root.join("bin").join(n);
            json!({"sha256":digest(&p),"path":p,"version_args":["--version"]})
        })
        .collect();
    let gate = PathBuf::from(env!("CARGO_BIN_EXE_lustr-bench-gate"));
    tools.push(json!({"sha256":digest(&gate),"path":gate,"version_args":["--version"]}));
    let coordinator = root.join("bin/lustr");
    let cargo = root.join("bin/cargo");
    let rustc = root.join("bin/rustc");
    let mut lanes = Vec::new();
    for name in ["baseline", "control"] {
        let target = out.join("passed/targets").join(name);
        lanes.push(json!({"name":name,"env":{"PATH":format!("{}:/usr/bin:/bin",root.join("bin").display()),"HOME":out,"CARGO_HOME":out.join("cargo-home"),"RUSTC":rustc,"RUSTDOC":root.join("bin/rustdoc"),"CARGO":cargo,"CARGO_BUILD_JOBS":"2","CARGO_NET_OFFLINE":"true","CARGO_TERM_COLOR":"never"},"tools":tools,
            "command":cmd(json!([coordinator,"--bluefe-root",root,"check","--locked","--offline","-vv"]),0),
            "selection":cmd(json!([cargo,"metadata","--format-version","1","--locked","--offline"]),0),
            "evidence":cmd(json!([coordinator,"--bluefe-root",root,"check","--locked","--offline","-vv"]),0),
            "evidence_contains":["--edition=2024","-C debuginfo=2"],
            "gates":[{"name":"run-and-unwind","kind":"runtime","compare":"stdout","command":cmd(json!([coordinator,"--bluefe-root",root,"run","--quiet","--locked","--offline"]),0)},
                {"name":"executed-tests","kind":"tests","compare":"stdout","command":cmd(json!([gate,"tests","test","--locked","--offline"]),0)},
                {"name":"invalid-borrow","kind":"rejection","compare":"stdout","command":cmd(json!([gate,"reject","reject.rs"]),1)}],
            "normalize":[[target,"<TARGET>"]]}));
    }
    let spec = json!({"schema":1,"source":fixture,"repository":"https://github.com/glopyglerky/lustr","revision":"fixture bytes in runner source manifest","package":"bench-tiny","root_crate":"bench_tiny","profile":"dev","target":"x86_64-unknown-linux-gnu","features":[],"semantic_settings":{"panic":"unwind","debug":"2","overflow-checks":"true"},"external_inputs":[],"scenario":"primed-root-check","force_root":"src/main.rs","snapshot_mtime_authorized":true,"pairs":8,"warmup_pairs":1,"seed":6757,"bootstrap_resamples":1000,"timeout_seconds":30,"output_limit_bytes":1048576,"lanes":lanes});
    let path = out.join("spec.json");
    fs::write(&path, serde_json::to_vec_pretty(&spec).unwrap()).unwrap();
    let exe = env!("CARGO_BIN_EXE_lustr-bench");
    let status = Command::new(exe).arg(&path).arg(out.join("passed")).status().unwrap();
    assert!(status.success());
    let report: Value = serde_json::from_slice(&fs::read(out.join("passed/report.json")).unwrap()).unwrap();
    assert_eq!(report["status"], "passed");
    assert_eq!(report["summaries"][0]["analysis"]["pairs"], 8);
    assert_eq!(report["source_before"], report["source_after"]);
    assert_eq!(report["records"].as_array().unwrap().iter().filter(|r| r["phase"] == "timed").count(), 16);
    assert!(!Command::new(exe).arg(&path).arg(out.join("passed")).status().unwrap().success());
    // Profile values cannot hide a feature difference from operation comparison.
    let mut unequal = spec.clone();
    for key in ["command", "evidence"] {
        unequal["lanes"][0][key]["argv"] =
            json!([coordinator, "--bluefe-root", root, "rustc", "--profile", "check", "--locked", "--offline", "-vv"]);
        unequal["lanes"][1][key]["argv"] = json!([
            coordinator,
            "--bluefe-root",
            root,
            "rustc",
            "--features",
            "extra",
            "--profile",
            "check",
            "--locked",
            "--offline",
            "-vv"
        ]);
    }
    fs::write(out.join("unequal.json"), serde_json::to_vec(&unequal).unwrap()).unwrap();
    assert!(!Command::new(exe).arg(out.join("unequal.json")).arg(out.join("unequal")).status().unwrap().success());
    assert!(!out.join("unequal").exists());
    // Failed correctness must leave raw evidence and no statistical conclusions.
    let mut bad = spec.clone();
    bad["lanes"][0]["gates"][0]["command"]["expected_exit"] = json!(9);
    bad["lanes"][1]["gates"][0]["command"]["expected_exit"] = json!(9);
    fs::write(out.join("bad.json"), serde_json::to_vec(&bad).unwrap()).unwrap();
    assert!(!Command::new(exe).arg(out.join("bad.json")).arg(out.join("failed")).status().unwrap().success());
    let failed: Value = serde_json::from_slice(&fs::read(out.join("failed/report.json")).unwrap()).unwrap();
    assert_eq!(failed["status"], "failed");
    assert!(failed["summaries"].is_null());
    // Misleading diagnostic text must fail qualification before timed samples.
    for (case, name, text) in [
        ("filename", "E0502.rs", "fn main() { let _: u32 = \"wrong type\"; }\n"),
        ("snippet", "snippet.rs", "fn main() { let _: u32 = \"E0502\"; }\n"),
    ] {
        let path = fixture.join(name);
        fs::write(&path, text).unwrap();
        let mut misleading = spec.clone();
        for lane in misleading["lanes"].as_array_mut().unwrap() {
            lane["gates"][2]["command"]["argv"] = json!([gate, "reject", path]);
        }
        let input = out.join(format!("diagnostic-{case}.json"));
        let report_dir = out.join(format!("diagnostic-{case}"));
        fs::write(&input, serde_json::to_vec(&misleading).unwrap()).unwrap();
        assert!(!Command::new(exe).arg(&input).arg(&report_dir).status().unwrap().success());
        let refused: Value = serde_json::from_slice(&fs::read(report_dir.join("report.json")).unwrap()).unwrap();
        assert!(refused["summaries"].is_null());
        let records = refused["records"].as_array().unwrap();
        assert!(records.iter().all(|r| r["phase"] != "timed"));
        assert!(records.iter().any(|r| r["phase"] == "gate:invalid-borrow" && r["exit"] == 42));
    }
    // Source-configured shared intermediates are refused before any compiler command.
    fs::create_dir(fixture.join(".cargo")).unwrap();
    fs::write(
        fixture.join(".cargo/config.toml"),
        format!("[build]\nbuild-dir = {:?}\n", out.join("shared-intermediates").to_str().unwrap()),
    )
    .unwrap();
    let mut clean = spec.clone();
    clean["scenario"] = json!("clean-dependency-build");
    clean["force_root"] = Value::Null;
    for lane in clean["lanes"].as_array_mut().unwrap() {
        for key in ["command", "evidence"] {
            lane[key]["argv"] = json!([coordinator, "--bluefe-root", root, "build", "--locked", "--offline", "-vv"]);
        }
    }
    fs::write(out.join("shared.json"), serde_json::to_vec(&clean).unwrap()).unwrap();
    assert!(!Command::new(exe).arg(out.join("shared.json")).arg(out.join("shared")).status().unwrap().success());
    let shared: Value = serde_json::from_slice(&fs::read(out.join("shared/report.json")).unwrap()).unwrap();
    assert!(shared["summaries"].is_null());
    assert!(shared["error"].as_str().unwrap().contains("build_directory"));
    assert!(shared["records"].as_array().unwrap().iter().all(|r| r["phase"] == "identity" || r["phase"] == "dependency-selection"));
    // A different compiler diagnostic cannot satisfy the expected invalid-borrow gate.
    fs::write(fixture.join("reject.rs"), "fn main() { let _: u32 = \"wrong type\"; }\n").unwrap();
    let wrong = Command::new(&gate)
        .arg("reject")
        .arg(fixture.join("reject.rs"))
        .env("RUSTC", &rustc)
        .env("CARGO_TARGET_DIR", out.join("passed/targets/baseline"))
        .output()
        .unwrap();
    assert_eq!(wrong.status.code(), Some(42));
}

#[test]
fn rejection_codes_cannot_be_spoofed_by_paths_or_source_text() {
    let Some(root) = std::env::var_os("LUSTR_BENCH_TEST_BLUEFE_ROOT").map(PathBuf::from) else {
        return;
    };
    let out = PathBuf::from(std::env::var_os("LUSTR_BENCH_TEST_OUTPUT").expect("explicit output")).with_extension("diagnostic-cases");
    fs::create_dir(&out).unwrap();
    for (name, source) in
        [("E0502.rs", "fn main() { let _: u32 = \"wrong type\"; }\n"), ("snippet.rs", "fn main() { let _: u32 = \"E0502\"; }\n")]
    {
        let path = out.join(name);
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_lustr-bench-gate"))
            .args(["reject".as_ref(), path.as_os_str()])
            .env("RUSTC", root.join("bin/rustc"))
            .env("CARGO_TARGET_DIR", &out)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(42), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stderr).contains("E0308"));
    }
}
