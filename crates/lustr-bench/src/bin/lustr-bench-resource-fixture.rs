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
