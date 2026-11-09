//! Synthetic resource-protocol fixture, never a compiler benchmark.
#![forbid(unsafe_code)]
use std::{
    hint::black_box,
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    process::Command,
};
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    match args.get(1).and_then(|x| x.to_str()).unwrap_or("") {
        "large" | "small" => {
            let size = if args[1] == "large" { 64 * 1024 * 1024 } else { 1024 * 1024 };
            let mut memory = vec![0u8; size];
            for (i, value) in memory.iter_mut().enumerate() {
                *value = black_box(i as u8);
            }
            black_box(&memory);
            let start = std::time::Instant::now();
            let mut sum = 1u64;
            while start.elapsed().as_millis() < 25 {
                sum = black_box(sum.wrapping_mul(17).wrapping_add(3));
            }
            black_box(sum);
        }
        "bytes" => {
            assert_eq!(std::env::var_os("PAYLOAD_BYTES").unwrap().as_bytes(), b"\xff");
            assert_eq!(args[2].as_bytes(), b"\xfe");
            let mut input = Vec::new();
            std::io::stdin().read_to_end(&mut input).unwrap();
            assert!(input.is_empty());
            std::io::stdout().write_all(b"\0\xffout").unwrap();
            std::io::stderr().write_all(b"\xfeerr\0").unwrap();
        }
        "failed" => std::process::exit(7),
        "signal" => {
            nix::sys::signal::raise(nix::sys::signal::Signal::SIGTERM).unwrap();
        }
        "signal-unblock" => {
            assert!(nix::sys::signal::SigSet::thread_get_mask().unwrap().contains(nix::sys::signal::Signal::SIGTERM));
            let mut signals = nix::sys::signal::SigSet::empty();
            signals.add(nix::sys::signal::Signal::SIGTERM);
            signals.thread_unblock().unwrap();
            nix::sys::signal::raise(nix::sys::signal::Signal::SIGTERM).unwrap();
        }
        "collector-misuse" => {
            // This fixture leads an isolated test group. Its child collector does not.
            let mut sentinel = Command::new(&args[0]).arg("sleep").spawn().unwrap();
            let result = Command::new(&args[2])
                .arg("--internal-resource-collector-v2")
                .args(&args[3..5])
                .stdin(std::process::Stdio::null())
                .output();
            let alive = sentinel.try_wait().unwrap().is_none();
            let _ = sentinel.kill();
            sentinel.wait().unwrap();
            let result = result.unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&result.stderr).contains("collector must lead its process group"));
            assert!(alive, "collector signalled its sibling");
        }
        "sleep" => std::thread::sleep(std::time::Duration::from_secs(30)),
        "output" => loop {
            std::io::stdout().write_all(&[42; 4096]).unwrap();
        },
        "waited" => {
            assert!(Command::new(&args[0]).arg("large").status().unwrap().success());
        }
        _ => panic!("unknown fixture"),
    }
}
