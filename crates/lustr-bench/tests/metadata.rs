//! Opt-in real-Cargo schema compatibility and material-difference regression.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn digest(path: &Path) -> String {
    Sha256::digest(fs::read(path).unwrap()).iter().map(|b| format!("{b:02x}")).collect()
}

fn replace_paths(value: &mut Value, root: &str, out: &str, gate: &str) {
    match value {
        Value::String(text) => {
            *text = text.replace("/experiment/tools/lustr-bench-gate", gate).replace("/toolchain", root).replace("/experiment", out)
        }
        Value::Array(values) => {
            for value in values {
                replace_paths(value, root, out, gate);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                replace_paths(value, root, out, gate);
            }
        }
        _ => (),
    }
}

#[test]
fn real_cargo_unset_hint_matches_but_numeric_hint_does_not() {
    let Some(root) = std::env::var_os("LUSTR_BENCH_TEST_BLUEFE_ROOT").map(PathBuf::from) else {
        return;
    };
    let Some(stock) = std::env::var_os("LUSTR_BENCH_TEST_STOCK_CARGO").map(PathBuf::from) else {
        return;
    };
    let out = PathBuf::from(std::env::var_os("LUSTR_BENCH_TEST_OUTPUT").expect("explicit output")).with_extension("metadata-cases");
    fs::create_dir(&out).unwrap();
    let fixture = out.join("snapshot");
    fs::create_dir_all(fixture.join("src")).unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/tiny");
    for file in ["Cargo.toml", "Cargo.lock", "src/main.rs", "reject.rs"] {
        fs::copy(source.join(file), fixture.join(file)).unwrap();
    }
    let manifest = fixture.join("Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, format!("{original}\n[hints]\nmostly-unused = true\n")).unwrap();
    let gate = PathBuf::from(env!("CARGO_BIN_EXE_lustr-bench-gate"));
    let mut spec: Value = serde_json::from_str(include_str!("../sample-spec.json")).unwrap();
    replace_paths(&mut spec, root.to_str().unwrap(), out.to_str().unwrap(), gate.to_str().unwrap());
    let mut tools: Vec<_> = ["cargo", "rustc", "rustdoc", "lustr"]
        .into_iter()
        .map(|name| root.join("bin").join(name))
        .chain([gate, stock.clone()])
        .map(|path| json!({"sha256":digest(&path),"path":path,"version_args":["--version"]}))
        .collect();
    tools.dedup_by(|a, b| a["path"] == b["path"]);
    for (i, lane) in spec["lanes"].as_array_mut().unwrap().iter_mut().enumerate() {
        let cargo = if i == 0 { stock.clone() } else { root.join("bin/cargo") };
        lane["env"]["CARGO"] = json!(cargo);
        lane["tools"] = json!(tools);
        for key in ["command", "evidence"] {
            lane[key]["argv"] = json!([cargo, "check", "--locked", "--offline", "-vv"]);
        }
        lane["selection"]["argv"][0] = json!(cargo);
    }
    let executable = env!("CARGO_BIN_EXE_lustr-bench");
    let input = out.join("unset.json");
    fs::write(&input, serde_json::to_vec(&spec).unwrap()).unwrap();
    assert!(
        Command::new(executable).arg(&input).arg(out.join("passed")).status().unwrap().success(),
        "an unset optional hint must compare equal across the two real Cargo versions"
    );
    let report: Value = serde_json::from_slice(&fs::read(out.join("passed/report.json")).unwrap()).unwrap();
    assert_eq!(report["status"], "passed");
    assert_eq!(report["metadata_comparison_rule"], "cargo-optional-min-opt-level-none-v1");
    // A real non-null hint remains visible even when a Cargo version ignores it.
    fs::write(&manifest, format!("{original}\n[hints]\nmostly-unused = true\nmin-opt-level = 2\n")).unwrap();
    for lane in spec["lanes"].as_array_mut().unwrap() {
        lane["normalize"][0][0] = json!(out.join("material/targets").join(lane["name"].as_str().unwrap()));
    }
    fs::write(out.join("material.json"), serde_json::to_vec(&spec).unwrap()).unwrap();
    assert!(!Command::new(executable).arg(out.join("material.json")).arg(out.join("material")).status().unwrap().success());
    let report: Value = serde_json::from_slice(&fs::read(out.join("material/report.json")).unwrap()).unwrap();
    assert!(report["summaries"].is_null());
    assert!(report["error"].as_str().unwrap().contains("dependency selection differs"));
    assert!(report["records"].as_array().unwrap().iter().all(|r| r["phase"] != "timed"));
}
