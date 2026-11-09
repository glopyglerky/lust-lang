//! Real Linux collector protocol, explicitly bounded synthetic work only.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::process::{CommandExt, ExitStatusExt},
    path::Path,
    process::{Command, Stdio},
};
fn digest(p: &Path) -> String {
    Sha256::digest(fs::read(p).unwrap()).iter().map(|b| format!("{b:02x}")).collect()
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
            .arg("--internal-resource-collector-v1")
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
        assert_eq!(u["request_sha256"], digest(&input));
        assert_eq!(u["collector_sha256"], digest(runner));
        assert!(u["user_microseconds"].as_i64().unwrap() >= 0);
        assert!(u["system_microseconds"].as_i64().unwrap() >= 0);
        match *mode {
            "failed" => {
                assert_eq!(result.status.code(), Some(7));
                assert_eq!(u["raw_status"], 7 << 8);
            }
            "signal" => {
                assert_eq!(result.status.signal(), Some(15));
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
            .arg("--internal-resource-collector-v1")
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
