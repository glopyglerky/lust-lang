//! Real Linux collector protocol, explicitly bounded synthetic work only.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    os::unix::process::{CommandExt, ExitStatusExt},
    path::Path,
    process::{Command, Stdio},
};
fn digest(p: &Path) -> String {
    Sha256::digest(fs::read(p).unwrap()).iter().map(|b| format!("{b:02x}")).collect()
}

struct OwnedGroup(std::process::Child);
impl Drop for OwnedGroup {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = nix::sys::signal::killpg(nix::unistd::Pid::from_raw(self.0.id() as i32), nix::sys::signal::Signal::SIGKILL);
            let _ = self.0.wait();
        }
    }
}
impl OwnedGroup {
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(std::time::Instant::now() < deadline, "owned test group timed out");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

#[test]
fn collector_completion_preserves_sigpipe_blocked_signals_and_siblings() {
    let Some(base) = std::env::var_os("LUSTR_BENCH_RESOURCE_TEST_OUTPUT") else {
        return;
    };
    let root = Path::new(&base).with_extension("signals");
    fs::create_dir(&root).unwrap();
    let toolchain = std::env::var_os("LUSTR_BENCH_TEST_BLUEFE_ROOT").expect("explicit qualified nightly toolchain");
    let runner = Path::new(env!("CARGO_BIN_EXE_lustr-bench"));
    let fixture = Path::new(env!("CARGO_BIN_EXE_lustr-bench-resource-fixture"));
    let source = root.join("pipe.rs");
    let binary = root.join("pipe");
    // The selected compiler makes this Rust payload restore default SIGPIPE handling.
    // A file barrier lets the parent close stdout before the payload writes to it.
    fs::write(
        &source,
        r#"fn main() {
        use std::io::Write;
        let mut args = std::env::args_os().skip(1);
        std::fs::write(args.next().unwrap(), b"ready").unwrap();
        let go = args.next().unwrap();
        while !std::path::Path::new(&go).exists() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::io::stdout().write_all(b"pipe").unwrap();
    }"#,
    )
    .unwrap();
    let compiled = Command::new(Path::new(&toolchain).join("bin/rustc"))
        .args(["--crate-name", "resource_pipe", "--edition=2024", "-Zon-broken-pipe=kill", "--sysroot"])
        .arg(&toolchain)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));

    let request = |name: &str, argv: Value| {
        let input = root.join(format!("{name}.request.json"));
        let report = root.join(format!("{name}.usage.json"));
        fs::write(
            &input,
            serde_json::to_vec(&json!({"argv":argv,"env":[],"cwd":root,"collector_sha256":digest(runner),"report_path":report})).unwrap(),
        )
        .unwrap();
        (input, report)
    };
    let ready = root.join("ready");
    let go = root.join("go");
    let (input, report) = request("pipe", json!([binary, ready, go]));
    let mut group = OwnedGroup(
        Command::new(runner)
            .arg("--internal-resource-collector-v2")
            .arg(input)
            .arg(&report)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let lease = group.0.stdin.take().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !ready.exists() {
        assert!(group.0.try_wait().unwrap().is_none(), "collector exited before the file barrier");
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    drop(group.0.stdout.take());
    fs::write(&go, b"go").unwrap();
    assert!(group.wait().success());
    drop(lease);
    let usage: Value = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
    assert_eq!(usage["schema"], 2);
    assert_eq!(usage["raw_status"], 13);

    let (input, report) = request("blocked", json!([fixture, "signal-unblock"]));
    let old_mask = nix::sys::signal::SigSet::thread_get_mask().unwrap();
    let mut mask = nix::sys::signal::SigSet::empty();
    mask.add(nix::sys::signal::Signal::SIGTERM);
    mask.thread_block().unwrap();
    let spawned = Command::new(runner)
        .arg("--internal-resource-collector-v2")
        .arg(input)
        .arg(&report)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .process_group(0)
        .spawn();
    old_mask.thread_set_mask().unwrap();
    let mut group = OwnedGroup(spawned.unwrap());
    let lease = group.0.stdin.take().unwrap();
    assert!(group.wait().success());
    drop(lease);
    let usage: Value = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
    assert_eq!(usage["raw_status"], 15);

    let (input, report) = request("misuse", json!([fixture, "sleep"]));
    // Only this owned fixture and its children share the group; the test harness does not.
    let mut group = OwnedGroup(
        Command::new(fixture)
            .arg("collector-misuse")
            .arg(runner)
            .arg(input)
            .arg(&report)
            .env_clear()
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let status = group.wait();
    let mut stderr = Vec::new();
    group.0.stderr.take().unwrap().read_to_end(&mut stderr).unwrap();
    assert!(status.success(), "{}", String::from_utf8_lossy(&stderr));
    assert!(!report.exists());
}
#[test]
fn fresh_collectors_preserve_payload_and_do_not_accumulate_maxrss() {
    let Some(root) = std::env::var_os("LUSTR_BENCH_RESOURCE_TEST_OUTPUT") else {
        return;
    };
    let root = Path::new(&root);
    fs::create_dir(root).unwrap();
    let runner = Path::new(env!("CARGO_BIN_EXE_lustr-bench"));
    let fixture = env!("CARGO_BIN_EXE_lustr-bench-resource-fixture");
    let mut rss = Vec::new();
    for (id, mode) in ["large", "small", "failed", "signal", "bytes", "waited", "sleep", "output"].iter().enumerate() {
        let input = root.join(format!("{id}.request.json"));
        let output = root.join(format!("{id}.usage.json"));
        let mut argv = json!([fixture, mode]);
        if *mode == "bytes" {
            argv.as_array_mut().unwrap().push(json!({"hex":"fe"}));
        }
        let req =
            json!({"argv":argv,"env":[["PAYLOAD_BYTES",{"hex":"ff"}]],"cwd":root,"collector_sha256":digest(runner),"report_path":output});
        fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
        let mut child = Command::new(runner)
            .arg("--internal-resource-collector-v2")
            .arg(&input)
            .arg(&output)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .unwrap();
        let lease = child.stdin.take().unwrap();
        if ["sleep", "output"].contains(mode) {
            // Parent cancellation/lease loss must kill the collector and payload group.
            std::thread::sleep(std::time::Duration::from_millis(100));
            drop(lease);
            assert_eq!(child.wait().unwrap().signal(), Some(9));
            assert!(!output.exists());
            continue;
        }
        let result = child.wait_with_output().unwrap();
        drop(lease);
        let u: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert!(result.status.success(), "collector completion: {}", String::from_utf8_lossy(&result.stderr));
        assert_eq!(u["schema"], 2);
        assert_eq!(u["request_sha256"], digest(&input));
        assert_eq!(u["collector_sha256"], digest(runner));
        assert!(u["user_microseconds"].as_i64().unwrap() >= 0);
        assert!(u["system_microseconds"].as_i64().unwrap() >= 0);
        match *mode {
            "failed" => {
                assert_eq!(u["raw_status"], 7 << 8);
            }
            "signal" => {
                assert_eq!(u["raw_status"], 15);
            }
            "bytes" => {
                assert!(result.status.success());
                assert_eq!(result.stdout, b"\0\xffout");
                assert_eq!(result.stderr, b"\xfeerr\0");
            }
            _ => {
                assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
                assert!(u["user_microseconds"].as_i64().unwrap() > 0);
            }
        }
        if id < 2 {
            rss.push(u["peak_rss_kib"].as_u64().unwrap());
        }
        if *mode == "waited" {
            assert!(u["peak_rss_kib"].as_u64().unwrap() > 48 * 1024);
        }
        // Fresh-only publication: replay fails, preserving the original report bytes.
        let before = fs::read(&output).unwrap();
        let mut replay = Command::new(runner)
            .arg("--internal-resource-collector-v2")
            .arg(&input)
            .arg(&output)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap();
        let lease = replay.stdin.take().unwrap();
        assert!(!replay.wait().unwrap().success());
        drop(lease);
        assert_eq!(fs::read(output).unwrap(), before);
    }
    assert!(rss[0] > 48 * 1024);
    assert!(rss[1] < rss[0] / 2, "large then small: {rss:?}");
}
