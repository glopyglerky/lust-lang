use std::path::PathBuf;
use std::process::Command;

const COMMIT: &str = "2d20b42b883dcfc014f3c2c56081eeb6091c2179";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = std::env::var_os("RUSTC").expect("Cargo must set RUSTC");
    let query = |args: &[&str]| {
        let output = Command::new(&rustc).args(args).output().expect("query build compiler");
        assert!(output.status.success(), "build compiler query failed");
        String::from_utf8(output.stdout).expect("build compiler response must be UTF-8")
    };
    let version = query(&["-vV"]);
    assert!(
        version.lines().any(|line| line == format!("commit-hash: {COMMIT}")),
        "lustr-semantic requires nightly-2026-11-05, rustc commit {COMMIT}; got {version}"
    );
    let host = version.lines().find_map(|line| line.strip_prefix("host: ")).expect("compiler host");
    let release = version.lines().find_map(|line| line.strip_prefix("release: ")).expect("compiler release");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_HOST={host}");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_RELEASE={release}");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_COMMIT={COMMIT}");
    let sysroot = PathBuf::from(query(&["--print", "sysroot"]).trim_end_matches('\n'));
    let sysroot = sysroot.canonicalize().expect("compiler sysroot must exist");
    println!("cargo:rustc-env=LUSTR_BUILD_SYSROOT={}", sysroot.to_str().expect("toolchain location must be UTF-8"));
    // rustc-dev supplies rustc_driver as a dynamic library. The probe is an
    // installed host tool, so it must also find that compiler's shared runtime.
    if host.contains("linux") || host.contains("darwin") {
        for directory in [sysroot.join("lib"), sysroot.join("lib/rustlib").join(host).join("lib")] {
            let directory = directory.to_str().expect("toolchain location must be UTF-8");
            assert!(!directory.contains(['\n', '\r', ',']), "unsupported toolchain path for linker rpath");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        }
    }
}
