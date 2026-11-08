use std::process::{Command, exit};

fn main() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(arguments.len(), 3, "expected Cargo executable, child manifest, and child target directory");

    // Deliberately inherit the environment Cargo gave this application. Backend
    // profile settings introduced by the outer coordinator must not reach this
    // ordinary nested Cargo invocation, which has no unstable feature flags.
    let status = Command::new(&arguments[0])
        .args(["check", "--offline", "--locked", "--quiet", "--manifest-path"])
        .arg(&arguments[1])
        .arg("--target-dir")
        .arg(&arguments[2])
        .status()
        .expect("could not start ordinary nested Cargo");
    if !status.success() {
        eprintln!("ordinary nested Cargo check failed: {status}");
        exit(status.code().unwrap_or(1));
    }

    // The harness clears caller-provided backend values before the outer run.
    // A globally enabled unstable-feature workaround must not hide leaked values.
    for name in ["CARGO_PROFILE_DEV_CODEGEN_BACKEND", "CARGO_PROFILE_TEST_CODEGEN_BACKEND"] {
        assert!(std::env::var_os(name).is_none(), "backend configuration leaked into the application: {name}");
    }
    println!("nested Cargo check passed");
}
