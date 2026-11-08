//! Exercise Cargo's actual extension lookup and the direct lustr command.
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("lustr entrypoints {} {nonce}", std::process::id()));
        fs::create_dir_all(path.join("src")).unwrap();
        fs::create_dir_all(path.join("bin")).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cargo_extensions_and_direct_lustr_preserve_the_real_program_result() {
    let scratch = Scratch::new();
    fs::write(
        scratch.0.join("Cargo.toml"),
        "[package]\nname = \"lustr-entrypoint-probe\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        scratch.0.join("src/main.rs"),
        "fn main() { println!(\"{}\", std::env::args().skip(1).collect::<Vec<_>>().join(\"|\")); std::process::exit(23); }\n",
    )
    .unwrap();
    for (name, binary) in [("cargo-lust", env!("CARGO_BIN_EXE_cargo-lust")), ("cargo-lustr", env!("CARGO_BIN_EXE_cargo-lustr"))] {
        fs::copy(binary, scratch.0.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))).unwrap();
    }
    let mut paths = vec![scratch.0.join("bin")];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
    let path = std::env::join_paths(paths).unwrap();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let run = |program: &std::ffi::OsStr, prefix: &[&str]| -> Output {
        Command::new(program)
            .args(prefix)
            .args(["run", "--offline", "--quiet", "--", "--release", "argument with spaces", "lustr"])
            .current_dir(&scratch.0)
            .env("PATH", &path)
            .env("CARGO_TARGET_DIR", scratch.0.join("target"))
            .env("LUST_CARGO", &cargo)
            .env_remove("LUST_CRANELIFT_BACKEND")
            .output()
            .unwrap()
    };
    let baseline = run(&cargo, &[]);
    assert_eq!(baseline.status.code(), Some(23), "{}", String::from_utf8_lossy(&baseline.stderr));
    assert_eq!(baseline.stdout, b"--release|argument with spaces|lustr\n");
    for (program, prefix) in [
        (std::ffi::OsStr::new(env!("CARGO_BIN_EXE_lustr")), Vec::new()),
        (cargo.as_os_str(), vec!["lustr"]),
        (cargo.as_os_str(), vec!["lust"]),
    ] {
        let result = run(program, &prefix);
        assert_eq!(result.status.code(), baseline.status.code(), "{program:?} {prefix:?}: {}", String::from_utf8_lossy(&result.stderr));
        assert_eq!(result.stdout, baseline.stdout, "{program:?} {prefix:?}");
        assert_eq!(result.stderr, baseline.stderr, "{program:?} {prefix:?}");
    }
}
