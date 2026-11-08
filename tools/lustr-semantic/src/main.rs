#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate serde_json;

mod collect;
mod contract;

use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rustc_data_structures::profiling::{TimePassesFormat, get_resident_set_size, print_time_passes_entry};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use serde_json::{Value, json};

const COMMIT: &str = env!("LUSTR_BUILD_RUSTC_COMMIT");
const HOST: &str = env!("LUSTR_BUILD_RUSTC_HOST");
const RELEASE: &str = env!("LUSTR_BUILD_RUSTC_RELEASE");
const SYSROOT: &str = env!("LUSTR_BUILD_SYSROOT");

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(compiler) = args.next() else {
        eprintln!(
            "usage: lustr-semantic <rustc> <rustc arguments...>\nUse RUSTC_WORKSPACE_WRAPPER with LUSTR_SEMANTIC_DIR set to a report directory."
        );
        return ExitCode::from(2);
    };
    if compiler == "--version" {
        println!("lustr-semantic {} rustc={COMMIT} host={HOST}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if compiler == "--validate" {
        let Some(path) = args.next().filter(|_| args.next().is_none()) else {
            eprintln!("usage: lustr-semantic --validate <report.json>");
            return ExitCode::from(2);
        };
        let validation = fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string()))
            .and_then(|report| contract::validate_envelope(&report));
        return match validation {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("lustr-semantic: incompatible inspection envelope: {error}");
                ExitCode::from(1)
            }
        };
    }
    let os_args: Vec<OsString> = std::iter::once(compiler.clone()).chain(args).collect();
    // rustc_driver accepts UTF-8 arguments. Preserve other OS representations
    // by delegating intact to rustc, without producing an inspection report.
    let Ok(args) = os_args.iter().cloned().map(OsString::into_string).collect::<Result<Vec<_>, _>>() else {
        return delegate(&compiler, &os_args[1..]);
    };
    match run(&compiler, &args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("lustr-semantic: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(compiler: &OsStr, args: &[String]) -> Result<ExitCode, String> {
    let start_time = Instant::now();
    let start_rss = get_resident_set_size();
    let version = query(compiler, &["-vV"])?;
    for (key, expected) in [("commit-hash", COMMIT), ("host", HOST), ("release", RELEASE)] {
        let actual = version.lines().find_map(|line| line.strip_prefix(&format!("{key}: ")));
        if actual != Some(expected) {
            return Err(format!("compiler ABI mismatch: expected {key}: {expected}, got {actual:?}; use nightly-2026-11-05"));
        }
    }
    let sysroot = PathBuf::from(query(compiler, &["--print", "sysroot"])?.trim_end_matches(['\n', '\r']));
    verify_sysroot(&sysroot)?;
    verify_compiler_entrypoint(compiler)?;
    let report_dir = std::env::var_os("LUSTR_SEMANTIC_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("set LUSTR_SEMANTIC_DIR to the directory for successful semantic reports")?;
    let mut callbacks = Probe {
        time_passes: None,
        sysroot,
        invocation: json!({
            "compiler": args[0],
            "arguments": &args[1..],
            "working_directory": os_value(std::env::current_dir().map_err(|error| error.to_string())?.as_os_str()),
        }),
        report: None,
    };
    let early_dcx = rustc_session::EarlyDiagCtxt::new(rustc_session::config::ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);
    rustc_driver::install_ice_hook("https://github.com/glopyglerky/lustr/issues", |_| ());
    rustc_driver::install_ctrlc_handler();
    let status = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(args, &mut callbacks));
    if let Some(format) = callbacks.time_passes {
        print_time_passes_entry("total", start_time.elapsed(), start_rss, get_resident_set_size(), format);
    }
    if status != ExitCode::SUCCESS {
        return Ok(status);
    }
    if let Some(mut report) = callbacks.report {
        // The callback precedes code generation. Only a successful return from
        // the complete compiler, including linking, authorizes publication.
        report["compilation_succeeded"] = Value::Bool(true);
        contract::validate_envelope(&report).map_err(|error| format!("invalid inspection envelope: {error}"))?;
        publish(&report_dir, &report).map_err(|error| format!("could not publish semantic report: {error}"))?;
    }
    Ok(status)
}

fn verify_sysroot(path: &Path) -> Result<(), String> {
    let actual = path.canonicalize().map_err(|error| format!("cannot verify compiler sysroot {path:?}: {error}"))?;
    if actual != Path::new(SYSROOT) {
        return Err(format!(
            "compiler ABI mismatch: sysroot {actual:?} differs from the installation used to build this probe, {SYSROOT:?}; rebuild the probe for that installation"
        ));
    }
    Ok(())
}

fn query(compiler: &OsStr, args: &[&str]) -> Result<String, String> {
    let output = Command::new(compiler).args(args).output().map_err(|error| format!("cannot query {compiler:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!("compiler identity query failed with {}: {}", output.status, String::from_utf8_lossy(&output.stderr)));
    }
    String::from_utf8(output.stdout).map_err(|_| "compiler identity is not UTF-8".to_owned())
}

fn verify_compiler_entrypoint(compiler: &OsStr) -> Result<(), String> {
    // Matching query output does not establish wrapper equivalence: an RUSTC
    // wrapper may inject flags only for compilation. The in-process driver can
    // replace only the actual pinned compiler, never an arbitrary executable.
    let supplied = Path::new(compiler);
    let resolved = if supplied.is_absolute() || supplied.components().count() > 1 {
        supplied.canonicalize().ok()
    } else {
        std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path)
                .map(|directory| directory.join(supplied))
                .find(|candidate| candidate.is_file())
                .and_then(|candidate| candidate.canonicalize().ok())
        })
    };
    let expected = Path::new(SYSROOT)
        .join("bin")
        .join(format!("rustc{}", std::env::consts::EXE_SUFFIX))
        .canonicalize()
        .map_err(|error| format!("cannot resolve pinned compiler executable: {error}"))?;
    if resolved.as_ref() != Some(&expected) {
        return Err(format!(
            "unsupported compiler entrypoint {compiler:?}: inspection requires the actual pinned executable {expected:?}; custom compiler wrappers and rustup proxies cannot be replaced in process. Set RUSTC to the result of `rustup which --toolchain nightly-2026-11-05 rustc`"
        ));
    }
    Ok(())
}

struct Probe {
    time_passes: Option<TimePassesFormat>,
    sysroot: PathBuf,
    invocation: Value,
    report: Option<Value>,
}

impl Callbacks for Probe {
    fn config(&mut self, config: &mut interface::Config) {
        // Match pinned rustc's TimePassesCallbacks and its main() epilogue.
        // Compiler --print queries must not acquire an extra timing record.
        self.time_passes = (config.opts.prints.is_empty() && config.opts.unstable_opts.time_passes)
            .then_some(config.opts.unstable_opts.time_passes_format);
        config.opts.trimmed_def_paths = true;
        // Recreate the supplied rustc executable's default. An explicit
        // --sysroot must also refer to this exact compiler installation.
        config.opts.sysroot.default = self.sysroot.clone();
        if let Err(error) = verify_sysroot(config.opts.sysroot.path()) {
            rustc_session::EarlyDiagCtxt::new(config.opts.error_format).early_fatal(error);
        }
    }

    fn after_analysis<'tcx>(&mut self, _compiler: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        if tcx.dcx().has_errors().is_some() {
            return Compilation::Continue;
        }
        // Inspection names are not user diagnostics. Avoid rustc's diagnostic
        // path-trimming query, which deliberately requires a diagnostic emission.
        let mut report = match rustc_middle::ty::print::with_no_trimmed_paths!(collect::report(tcx)) {
            Ok(report) => report,
            Err(error) => tcx.dcx().fatal(format!("lustr-semantic could not inspect checked compiler input: {error}")),
        };
        report["invocation"] = self.invocation.clone();
        report["compiler"] = json!({"commit": COMMIT, "host": HOST, "release": RELEASE, "sysroot": SYSROOT});
        self.report = Some(report);
        Compilation::Continue
    }
}

fn publish(directory: &Path, report: &Value) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
    let basename = format!("{}-{nonce}", std::process::id());
    let temporary = directory.join(format!(".lustr-semantic-{basename}.tmp"));
    let destination = directory.join(format!("lustr-semantic-{basename}.json"));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    let result = (|| {
        serde_json::to_writer_pretty(&mut file, report).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        // Hard-link publication is atomic and will not replace a prior report.
        fs::hard_link(&temporary, &destination)
    })();
    let cleanup = fs::remove_file(&temporary);
    result.and(cleanup)
}

fn os_value(value: &OsStr) -> Value {
    if let Some(value) = value.to_str() {
        return json!(value);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        json!({"unix_bytes_hex": value.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()})
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        json!({"windows_utf16": value.encode_wide().collect::<Vec<_>>()})
    }
    #[cfg(not(any(unix, windows)))]
    {
        json!({"encoded_os_bytes_hex": value.as_encoded_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()})
    }
}

#[cfg(unix)]
fn delegate(compiler: &OsStr, args: &[OsString]) -> ExitCode {
    use std::os::unix::process::CommandExt;
    let error = Command::new(compiler).args(args).exec();
    eprintln!("lustr-semantic: cannot execute {compiler:?}: {error}");
    ExitCode::from(if error.kind() == io::ErrorKind::NotFound { 127 } else { 126 })
}

#[cfg(not(unix))]
fn delegate(compiler: &OsStr, args: &[OsString]) -> ExitCode {
    match Command::new(compiler).args(args).status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(error) => {
            eprintln!("lustr-semantic: cannot execute {compiler:?}: {error}");
            ExitCode::from(if error.kind() == io::ErrorKind::NotFound { 127 } else { 126 })
        }
    }
}
