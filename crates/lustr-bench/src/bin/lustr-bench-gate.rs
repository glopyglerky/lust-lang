//! Normalize successful libtest summaries while retaining unmodified output.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    io::Write,
    path::Path,
    process::{Command, ExitCode},
};

fn test_summary(stdout: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(stdout).map_err(|_| "test output is not UTF-8")?;
    let mut groups = Vec::new();
    let mut names = Vec::new();
    for line in text.lines() {
        if let Some(result) = line.strip_prefix("test result: ok. ") {
            let counts = result.split("; finished in ").next().ok_or("missing test counts")?;
            groups.push(counts.to_owned());
        }
        if line.starts_with("test ") && line.ends_with(" ... ok") {
            names.push(line.to_owned());
        }
    }
    if groups.is_empty() || names.is_empty() {
        return Err("no completed, nonempty test harness observed".into());
    }
    groups.sort();
    names.sort();
    Ok(json!({"kind":"tests","groups":groups,"passed_test_names":names}))
}

fn has_error_code(stderr: &[u8], code: &str) -> bool {
    stderr.split(|byte| *byte == b'\n').any(|line| {
        serde_json::from_slice::<Value>(line).is_ok_and(|diagnostic| {
            diagnostic["$message_type"] == "diagnostic" && diagnostic["level"] == "error" && diagnostic["code"]["code"] == code
        })
    })
}

fn validate_tests(args: &[OsString]) -> Result<(), &'static str> {
    if args.first().is_none_or(|arg| arg != "test") || args.iter().any(|arg| arg == "--no-run") {
        return Err("tests gate requires an executing Cargo test command");
    }
    Ok(())
}

fn routed_tests(mut args: impl Iterator<Item = OsString>) -> Result<Command, Box<dyn std::error::Error>> {
    let coordinator = args.next().ok_or("missing coordinator executable")?;
    if !Path::new(&coordinator).is_absolute() {
        return Err("coordinator must be absolute".into());
    }
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--bluefe-root")) {
        return Err("routed tests require --bluefe-root".into());
    }
    let root = args.next().ok_or("missing toolchain root")?;
    if !Path::new(&root).is_absolute() {
        return Err("toolchain root must be absolute".into());
    }
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--engine")) {
        return Err("routed tests require an explicit --engine".into());
    }
    let engine = args.next().ok_or("missing engine")?;
    if !["auto", "rustc", "cranelift"].iter().any(|name| engine == *name) {
        return Err("routed tests require auto, rustc or cranelift".into());
    }
    let cargo_args: Vec<_> = args.collect();
    validate_tests(&cargo_args)?;
    for (variable, executable) in [("CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")] {
        let selected = std::env::var_os(variable).ok_or("explicit tool selection is required")?;
        if selected.as_os_str() != Path::new(&root).join("bin").join(executable).as_os_str() {
            return Err(format!("routed {variable} selection must match --bluefe-root").into());
        }
    }
    let mut command = Command::new(coordinator);
    command.arg("--bluefe-root").arg(root).arg("--engine").arg(engine).args(cargo_args);
    Ok(command)
}

fn execute() -> Result<u8, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let mode = args.next().ok_or("usage: lustr-bench-gate tests <Cargo args...> | tests-via <coordinator> --bluefe-root <root> --engine <engine> test <args...> | reject <source>")?;
    if mode == "--version" && args.next().is_none() {
        println!("lustr-bench-gate {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    let rejection = mode == "reject";
    let mut command = if mode == "tests-via" {
        routed_tests(args)?
    } else {
        let variable = if rejection {
            "RUSTC"
        } else if mode == "tests" {
            "CARGO"
        } else {
            return Err("unknown gate".into());
        };
        let program = std::env::var_os(variable).ok_or("explicit tool selection is required")?;
        if !Path::new(&program).is_absolute() {
            return Err("gate tool must be absolute".into());
        }
        let mut command = Command::new(program);
        if rejection {
            let source = args.next().ok_or("missing rejection source")?;
            if args.next().is_some() {
                return Err("unexpected rejection argument".into());
            }
            let target = std::env::var_os("CARGO_TARGET_DIR").ok_or("runner-owned output required")?;
            command.args([
                OsString::from("--edition=2024"),
                OsString::from("--error-format=json"),
                source,
                OsString::from("--emit=metadata"),
                OsString::from("--out-dir"),
                target,
            ]);
        } else {
            let args: Vec<_> = args.collect();
            validate_tests(&args)?;
            command.args(args);
        }
        command
    };
    let output = command.output()?;
    // Raw streams remain in the runner's evidence; only comparison output is normalized.
    let mut diagnostic = std::io::stderr().lock();
    writeln!(diagnostic, "raw status: {}", output.status)?;
    writeln!(diagnostic, "raw stdout: {} bytes", output.stdout.len())?;
    diagnostic.write_all(&output.stdout)?;
    writeln!(diagnostic, "\nraw stderr: {} bytes", output.stderr.len())?;
    diagnostic.write_all(&output.stderr)?;
    diagnostic.flush()?;
    if rejection {
        if output.status.code() != Some(1) || !has_error_code(&output.stderr, "E0502") {
            return Err("expected invalid-borrow diagnostic E0502 was not observed".into());
        }
        println!("{}", json!({"kind":"rejection","diagnostic":"E0502"}));
        Ok(1)
    } else {
        if !output.status.success() {
            return Err("project tests failed".into());
        }
        println!("{}", test_summary(&output.stdout)?);
        Ok(0)
    }
}

fn main() -> ExitCode {
    match execute() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("gate validation failed: {error}");
            ExitCode::from(42)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durations_are_ignored_but_counts_and_test_names_are_preserved() {
        let a = b"test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let b = b"test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.10s\n";
        assert_eq!(test_summary(a).unwrap(), test_summary(b).unwrap());
        let changed = b"test x ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        assert_ne!(test_summary(a).unwrap(), test_summary(changed).unwrap());
        assert!(test_summary(b"build succeeded").is_err());
    }
}
