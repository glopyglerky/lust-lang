use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};

/// Inspect installed tools. This never invokes a project build, installs a
/// component, changes a toolchain, or claims that availability is qualification.
pub fn run() -> i32 {
    let programs = cargo_lust::Programs::from_environment();
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let cargo = probe(&programs.cargo, &["--version"]);
    let compiler = probe(&rustc, &["--version", "--verbose"]);
    let lust = probe(&programs.lust, &["--version"]);
    let sysroot = probe(&rustc, &["--print", "sysroot"]);
    let host = compiler.stdout.lines().find_map(|line| line.strip_prefix("host: ")).unwrap_or("");
    let nightly = compiler
        .stdout
        .lines()
        .any(|line| line.strip_prefix("release: ").is_some_and(|release| release.contains("nightly") || release.contains("dev")));
    let backend_libraries =
        if sysroot.success && !host.is_empty() { backend_libraries(Path::new(sysroot.stdout.trim()), host) } else { Vec::new() };
    let ready = cargo.success && compiler.success;
    println!(
        "{{\"format_version\":1,\"cargo\":{},\"rustc\":{},\"lust\":{},\"cranelift\":{{\"nightly_compiler\":{},\"library_present\":{},\"libraries\":[{}],\"qualified\":false}},\"automatic_lust_routing\":false,\"notes\":[{},{}]}}",
        cargo.json(),
        compiler.json(),
        lust.json(),
        nightly,
        !backend_libraries.is_empty(),
        backend_libraries.iter().map(|name| json(name)).collect::<Vec<_>>().join(","),
        json("Availability is not semantic or performance qualification. Cranelift remains explicit and experimental."),
        json(
            "Compiler probes use PATH and RUSTC. Cargo configuration, custom wrappers, or LUST_CARGO may select a different compiler; this command does not build a project to resolve them."
        ),
    );
    i32::from(!ready)
}

fn backend_libraries(sysroot: &Path, host: &str) -> Vec<String> {
    let dir = sysroot.join("lib").join("rustlib").join(host).join("codegen-backends");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file() || kind.is_symlink()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_cranelift_library(name))
        .collect();
    names.sort();
    names
}

fn is_cranelift_library(name: &str) -> bool {
    name.contains("rustc_codegen_cranelift") && [".so", ".dylib", ".dll"].iter().any(|suffix| name.ends_with(suffix))
}

struct Probe {
    program: String,
    success: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Probe {
    fn json(&self) -> String {
        format!(
            "{{\"program\":{},\"success\":{},\"exit_code\":{},\"stdout\":{},\"stderr\":{}}}",
            json(&self.program),
            self.success,
            self.code.map_or_else(|| "null".to_owned(), |code| code.to_string()),
            json(&self.stdout),
            json(&self.stderr),
        )
    }
}

fn probe(program: &OsStr, args: &[&str]) -> Probe {
    match Command::new(program).args(args).output() {
        Ok(Output { status, stdout, stderr }) => Probe {
            program: program.to_string_lossy().into_owned(),
            success: status.success(),
            code: status.code(),
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        },
        Err(error) => Probe {
            program: program.to_string_lossy().into_owned(),
            success: false,
            code: None,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn json(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_native_backend_libraries_count() {
        assert!(is_cranelift_library("librustc_codegen_cranelift-123.so"));
        assert!(is_cranelift_library("librustc_codegen_cranelift.dylib"));
        assert!(is_cranelift_library("rustc_codegen_cranelift.dll"));
        assert!(!is_cranelift_library("rustc_codegen_cranelift.rlib"));
        assert!(!is_cranelift_library("librustc_codegen_llvm.so"));
    }

    #[test]
    fn missing_tools_are_reported_without_a_panic() {
        let absent = probe(OsStr::new("cargo-lust-test-executable-that-does-not-exist"), &["--version"]);
        assert!(!absent.success);
        assert!(absent.code.is_none());
        assert!(!absent.stderr.is_empty());
    }
}
