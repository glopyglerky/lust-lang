use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn native_cache_guard_preserves_child_inputs_output_failure_and_single_execution() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let scratch = Scratch(std::env::temp_dir().join(format!("lustr cache guard {} {nonce}", std::process::id())));
    fs::create_dir(&scratch.0).unwrap();
    let probe_source = scratch.0.join("probe.rs");
    fs::write(
        &probe_source,
        r#"
use std::io::Write;
fn main() {
    let expected = std::env::var("EXPECTED_CACHE_DISABLED").unwrap();
    assert_eq!(std::env::var("LUST_NO_CACHE").unwrap(), expected);
    assert_eq!(std::env::var("LUST_NO_DAEMON").unwrap(), expected);
    let mut calls = std::fs::OpenOptions::new().create(true).append(true)
        .open(std::env::var_os("CALLS_LOG").unwrap()).unwrap();
    writeln!(calls, "called").unwrap();
    println!("{} {:?}", std::env::var("LUSTR_CACHE_INPUT").unwrap(), std::env::args_os().skip(1).collect::<Vec<_>>());
    eprintln!("native failure retained");
    std::process::exit(37);
}
"#,
    )
    .unwrap();
    let probe = scratch.0.join(format!("native probe{}", std::env::consts::EXE_SUFFIX));
    let built = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg(&probe_source)
        .arg("--edition=2024")
        .arg("-o")
        .arg(&probe)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let source = scratch.0.join("source with spaces.rs");
    let calls = scratch.0.join("calls.log");
    let cases = [
        ("fn main() { println!(\"{}\", env /* gap */ ! (\"LUSTR_CACHE_INPUT\")); }", "1", false),
        ("fn main() { println!(\"42\"); }", "0", false),
        ("fn main() { println!(\"42\"); }", "1", true),
    ];
    for (index, (text, disabled, flags_first)) in cases.iter().enumerate() {
        fs::write(&source, text).unwrap();
        let mut args = vec![OsString::from("build")];
        if *flags_first {
            args.push("--json".into());
        }
        args.extend([source.clone().into_os_string(), "-o".into(), scratch.0.join("output with spaces").into_os_string()]);
        let value = format!("current-input-{index}");
        let output = Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
            .args(["--engine", "lust"])
            .args(&args)
            .env("LUSTC", &probe)
            .env("LUST_NO_CACHE", "0")
            .env("LUST_NO_DAEMON", "0")
            .env("EXPECTED_CACHE_DISABLED", disabled)
            .env("CALLS_LOG", &calls)
            .env("LUSTR_CACHE_INPUT", &value)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(37), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("{value} {args:?}\n"));
        assert_eq!(output.stderr, b"native failure retained\n");
        assert_eq!(fs::read(&source).unwrap(), text.as_bytes());
        assert_eq!(fs::read_to_string(&calls).unwrap(), "called\n".repeat(index + 1));
    }
    let planned = Command::new(env!("CARGO_BIN_EXE_cargo-lust"))
        .args(["--engine", "lust", "plan", "build"])
        .arg(scratch.0.join("absent.rs"))
        .env("LUSTC", &probe)
        .output()
        .unwrap();
    assert!(planned.status.success());
    assert!(planned.stderr.is_empty());
    assert!(String::from_utf8(planned.stdout).unwrap().contains("\"environment\":{}"));
    assert_eq!(fs::read_to_string(calls).unwrap(), "called\n".repeat(cases.len()));
}
