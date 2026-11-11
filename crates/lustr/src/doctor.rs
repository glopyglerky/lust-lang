use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};

/// Inspect installed tools. This never invokes a project build, installs a
/// component, changes a toolchain, or claims that availability is qualification.
pub(crate) fn run_for_root(root: Option<&Path>, toolchain: Option<&OsStr>) -> i32 {
    if root.is_some() && toolchain.is_some() {
        eprintln!("lustr: --bluefe-root cannot be combined with +toolchain");
        return 2;
    }
    let toolchain = match toolchain {
        Some(value) => match value.to_str().and_then(|name| name.strip_prefix('+')).filter(|name| !name.is_empty()) {
            Some(name) => Some(name),
            None => {
                eprintln!("lustr: invalid +toolchain");
                return 2;
            }
        },
        None => None,
    };
    let mut programs = crate::Programs::from_environment();
    if toolchain.is_some() && programs.explicit_cargo {
        eprintln!("lustr: +toolchain cannot be combined with LUST_CARGO; select either a rustup toolchain or a Cargo executable");
        return 2;
    }
    if let Some(root) = root {
        programs.cargo = crate::toolchain::program(root, "cargo").into_os_string();
    }
    let rustc = root
        .map(|root| crate::toolchain::program(root, "rustc").into_os_string())
        .unwrap_or_else(|| std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc")));
    let cargo = rust_probe("cargo", &programs.cargo, &["--version"], toolchain);
    let compiler = rust_probe("rustc", &rustc, &["--version", "--verbose"], toolchain);
    let rustdoc = root
        .map(|root| crate::toolchain::program(root, "rustdoc").into_os_string())
        .unwrap_or_else(|| std::env::var_os("RUSTDOC").unwrap_or_else(|| OsString::from("rustdoc")));
    let documentation = rust_probe("rustdoc", &rustdoc, &["--version", "--verbose"], toolchain);
    let lust = probe(&programs.lust, &["--version"]);
    let sysroot = rust_probe("rustc", &rustc, &["--print", "sysroot"], toolchain);
    let host = compiler.stdout.lines().find_map(|line| line.strip_prefix("host: ")).unwrap_or("");
    let nightly = compiler
        .stdout
        .lines()
        .any(|line| line.strip_prefix("release: ").is_some_and(|release| release.contains("nightly") || release.contains("dev")));
    let backend_libraries =
        if sysroot.success && !host.is_empty() { backend_libraries(Path::new(sysroot.stdout.trim()), host) } else { Vec::new() };
    let configured_backend = std::env::var_os("LUST_CRANELIFT_BACKEND")
        .map(|value| {
            let path = Path::new(&value);
            format!(
                "{{\"path\":{},\"absolute\":{},\"file_present\":{}}}",
                json(&value.to_string_lossy()),
                path.is_absolute(),
                path.is_file()
            )
        })
        .unwrap_or_else(|| "null".to_owned());
    let ready = cargo.success && compiler.success && ((root.is_none() && toolchain.is_none()) || documentation.success);
    println!(
        "{{\"format_version\":1,\"cargo\":{},\"rustc\":{},\"rustdoc\":{},\"sysroot\":{},\"lust\":{},\"cranelift\":{{\"nightly_compiler\":{},\"preview_library_present\":{},\"preview_libraries\":[{}],\"configured_backend\":{},\"qualified\":false}},\"automatic_lust_routing\":false,\"notes\":[{},{}]}}",
        cargo.json(),
        compiler.json(),
        documentation.json(),
        sysroot.json(),
        lust.json(),
        nightly,
        !backend_libraries.is_empty(),
        backend_libraries.iter().map(|name| json(name)).collect::<Vec<_>>().join(","),
        configured_backend,
        json(
            "Availability is not qualification. The stock preview lacks required unwind support; configure an explicitly built unwind-enabled backend with LUST_CRANELIFT_BACKEND."
        ),
        json(
            "Compiler probes use the selected +toolchain through rustup run without installation, or the explicit or discovered package root, otherwise PATH, RUSTC and RUSTDOC. Cargo configuration and custom wrappers may select a different compiler; this command does not build a project to resolve them."
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
    args: Vec<String>,
    success: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Probe {
    fn json(&self) -> String {
        format!(
            "{{\"program\":{},\"args\":[{}],\"success\":{},\"exit_code\":{},\"stdout\":{},\"stderr\":{}}}",
            json(&self.program),
            self.args.iter().map(|argument| json(argument)).collect::<Vec<_>>().join(","),
            self.success,
            self.code.map_or_else(|| "null".to_owned(), |code| code.to_string()),
            json(&self.stdout),
            json(&self.stderr),
        )
    }
}

fn probe(program: &OsStr, args: &[&str]) -> Probe {
    let mut command = Command::new(program);
    command.args(args);
    probe_command(command)
}

fn rust_probe(tool: &str, default: &OsStr, args: &[&str], toolchain: Option<&str>) -> Probe {
    if let Some(toolchain) = toolchain {
        let mut command = Command::new("rustup");
        command.args(["run", "--", toolchain, tool]).args(args).env_remove("CARGO");
        probe_command(command)
    } else {
        probe(default, args)
    }
}

fn probe_command(mut command: Command) -> Probe {
    let program = command.get_program().to_string_lossy().into_owned();
    let args = command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect();
    let (success, code, stdout, stderr) = match command.output() {
        Ok(Output { status, stdout, stderr }) => {
            (status.success(), status.code(), String::from_utf8_lossy(&stdout).into_owned(), String::from_utf8_lossy(&stderr).into_owned())
        }
        Err(error) => (false, None, String::new(), error.to_string()),
    };
    Probe { program, args, success, code, stdout, stderr }
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
