//! A native compiler entry point, not a compiler selector. Compilation calls
//! unchanged `driver::compile` in this process. No rustc fallback exists.
//!
//! Cargo protocol references, pinned to the bundled Cargo source:
//! https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/util/rustc.rs
//! https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_context/target_info.rs
//! https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/fingerprint/dep_info.rs

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use crate::protocol::{Client, Cmd, ProfileFlags, Request, SessionKey};

include!(concat!(env!("OUT_DIR"), "/native-env-inputs.rs"));

static CRATE_INPUT: std::sync::OnceLock<(PathBuf, String)> = std::sync::OnceLock::new();
pub(crate) fn crate_name(root: &Path) -> Option<&'static str> {
    CRATE_INPUT.get().filter(|(path, _)| path == root).map(|(_, name)| name.as_str())
}

#[derive(Default)]
struct Args {
    source: Option<PathBuf>,
    name: Option<String>,
    types: Vec<String>,
    prints: Vec<String>,
    edition: Option<String>,
    output: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    emits: Vec<String>,
    extra: String,
    metadata: String,
    features: Vec<String>,
    target: Option<String>,
    json: bool,
    artifacts: bool,
    version: bool,
    verbose: bool,
    debug: bool,
    panic_abort: bool,
    overflow: Option<bool>,
    debug_assertions: Option<bool>,
    incremental: Option<PathBuf>,
    strip_debug: bool,
    check_cfg: Vec<String>,
    warnings: bool,
}

fn value(args: &[String], at: &mut usize, flag: &str) -> Result<String, String> {
    *at += 1;
    args.get(*at).cloned().ok_or_else(|| format!("{flag} needs a value"))
}

fn boolean(value: &str) -> Result<bool, String> {
    match value {
        "yes" | "on" | "true" | "y" => Ok(true),
        "no" | "off" | "false" | "n" => Ok(false),
        _ => Err(format!("unsupported boolean value `{value}`")),
    }
}

fn parse(args: Vec<OsString>) -> Result<Args, String> {
    let args: Vec<String> = args.into_iter().map(|s| s.into_string()
        .map_err(|_| "this experimental native compiler requires UTF-8 arguments".to_string())).collect::<Result<_, _>>()?;
    let mut out = Args::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let (flag, attached) = arg.split_once('=').map_or((arg.as_str(), None), |(a,b)| (a,Some(b.to_string())));
        if attached.is_some() && matches!(flag, "-vV" | "-V" | "--version" | "-v" | "--verbose" | "-Wwarnings") {
            return Err(format!("{flag} does not take a value"));
        }
        let mut next = || attached.clone().map(Ok).unwrap_or_else(|| value(&args, &mut i, flag));
        match flag {
            "-vV" => { out.version = true; out.verbose = true; }
            "-V" | "--version" => out.version = true,
            "-v" | "--verbose" => out.verbose = true,
            "--crate-name" => out.name = Some(next()?),
            "--crate-type" => out.types.extend(next()?.split(',').map(str::to_string)),
            "--print" => out.prints.push(next()?),
            "--edition" => out.edition = Some(next()?),
            "--out-dir" => out.out_dir = Some(PathBuf::from(next()?)),
            "-o" => out.output = Some(PathBuf::from(next()?)),
            "--emit" => out.emits.extend(next()?.split(',').map(str::to_string)),
            "--target" => out.target = Some(next()?),
            "--error-format" => match next()?.as_str() {
                "json" => out.json = true,
                "human" => out.json = false,
                v => return Err(format!("unsupported error format `{v}`")),
            },
            "--json" => {
                for option in next()?.split(',') {
                    match option {
                        "diagnostic-rendered-ansi" | "future-incompat" => {},
                        "artifacts" => out.artifacts = true,
                        v => return Err(format!("unsupported JSON option `{v}`")),
                    }
                }
            }
            "--cfg" => {
                let cfg = next()?;
                let f = cfg.strip_prefix("feature=\"").and_then(|s| s.strip_suffix('"'))
                    .ok_or_else(|| format!("custom cfg `{cfg}` is not implemented in the native Cargo entry point"))?;
                if f.contains(['"','\\','\n','\r']) { return Err("unsupported feature spelling".into()); }
                out.features.push(f.to_string());
            }
            "--check-cfg" => out.check_cfg.push(next()?),
            "-L" => {
                let path = next()?;
                if !path.starts_with("dependency=") { return Err("native library search paths are not implemented".into()); }
                // Cargo always sends its dependency directory, including for a
                // dependency-free bin. External crate artifacts remain rejected.
            }
            "-Wwarnings" => out.warnings = true,
            "-C" | "--codegen" => codegen(&mut out, &next()?)?,
            _ if arg.starts_with("-C") => codegen(&mut out, &arg[2..])?,
            _ if arg.starts_with('-') && arg != "-" => return Err(format!("unsupported native rustc option `{arg}`")),
            _ => {
                if out.source.replace(PathBuf::from(arg)).is_some() { return Err("multiple input files are not supported".into()); }
            }
        }
        i += 1;
    }
    Ok(out)
}

fn codegen(out: &mut Args, text: &str) -> Result<(), String> {
    let (name, value) = text.split_once('=').ok_or_else(|| format!("codegen option `{text}` needs a value"))?;
    match name {
        "extra-filename" => {
            if value.contains(['/', '\\', '\n', '\r']) { return Err("invalid output suffix".into()); }
            out.extra = value.into();
        }
        "metadata" => out.metadata = value.into(),
        "opt-level" if value == "0" => {},
        // TinyCC emits no LLVM bitcode. Explicitly disabling it is satisfied;
        // requesting embedded LLVM data or LTO remains unsupported.
        "embed-bitcode" if !boolean(value)? => {},
        "debuginfo" if value == "0" => out.debug = false,
        "debuginfo" if value == "2" => out.debug = true,
        "strip" if value == "none" => out.strip_debug = false,
        "strip" if value == "debuginfo" => out.strip_debug = true,
        "panic" => out.panic_abort = match value {
            "abort" => true, "unwind" => false,
            _ => return Err(format!("unsupported panic strategy `{value}`")),
        },
        "overflow-checks" => out.overflow = Some(boolean(value)?),
        "debug-assertions" => out.debug_assertions = Some(boolean(value)?),
        "incremental" if !value.is_empty() => out.incremental = Some(PathBuf::from(value)),
        _ => return Err(format!("unsupported native codegen option `{text}`")),
    }
    Ok(())
}

fn profile(args: &Args) -> ProfileFlags {
    let debug_assertions = args.debug_assertions.unwrap_or(true);
    ProfileFlags { panic_abort: args.panic_abort,
        overflow_checks: args.overflow.unwrap_or(debug_assertions),
        debug_assertions }
}

fn show_version(verbose: bool) {
    println!("lustr-native {} (Lust 0.2.0, build {})", env!("CARGO_PKG_VERSION"), env!("LUST_BUILD_HASH"));
    if verbose {
        println!("binary: lustr-native\ncommit-hash: unknown\nhost: {}\nrelease: {}\nbackend: unchanged Lust C/TinyCC\nnative-full-rust: false", crate::protocol::host_triple(), env!("CARGO_PKG_VERSION"));
    }
}

fn print_info(args: &Args) -> Result<(), String> {
    let name = args.name.as_deref().unwrap_or("rust_out");
    for print in &args.prints {
        match print.as_str() {
            "file-names" => {
                let types = if args.types.is_empty() { vec!["bin".into()] } else { args.types.clone() };
                for ty in types {
                    if ty == "bin" { println!("{name}{}{}", args.extra, std::env::consts::EXE_SUFFIX); }
                    else { eprintln!("warning: unsupported crate type `{ty}` for lustr-native"); }
                }
            }
            "sysroot" => {
                let dir = crate::tcc::install_dir().map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
                println!("{}", dir.display());
            }
            "split-debuginfo" => println!("off"),
            "crate-name" => println!("{name}"),
            "cfg" => {
                let cfg = crate::resolve::cfg::CfgEnv::new(args.target.as_deref().unwrap_or(""), &args.features, false, profile(args));
                for flag in cfg.flags { println!("{flag}"); }
                for (key, value) in cfg.pairs { println!("{key}={}", crate::diag::json_str(&value)); }
                for feature in cfg.features { println!("feature={}", crate::diag::json_str(&feature)); }
            }
            _ => return Err(format!("unsupported native print request `{print}`")),
        }
    }
    Ok(())
}

fn escape_path(path: &Path) -> Result<String, String> {
    let s = path.to_str().ok_or("non-UTF-8 dependency path is not supported")?;
    // Cargo's current parser only unescapes spaces. Refuse paths that would
    // create malformed or ambiguous dep-info rather than forge freshness.
    if s.contains(['\n', '\r', '\t', '\\']) { return Err("dependency path cannot be represented safely in Cargo dep-info".into()); }
    Ok(s.replace(' ', "\\ "))
}

fn escape_env(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r")
}

fn dependency_info(session: &crate::driver::Session, source: &Path, output: &Path, path: &Path,
                   native_env: BTreeMap<String, Option<String>>) -> Result<(), String> {
    let mut files = BTreeSet::new();
    files.insert(source.to_path_buf());
    for i in 0..session.front.map.file_count() {
        let path = session.front.map.path(crate::diag::FileId(i as u32));
        if Path::new(path).is_file() { files.insert(PathBuf::from(path)); }
    }
    files.extend(session.front.resolve_cache.source_paths().filter(|p| p.is_file()).map(Path::to_path_buf));
    let mut envs = native_env;
    for chunk in &session.front.chunks {
        for dep in &chunk.expand_deps {
            match dep {
                crate::expand::Dep::File { path, .. } => { files.insert(path.clone()); }
                crate::expand::Dep::Env { name, value } => { envs.insert(name.clone(), value.clone()); }
                _ => {},
            }
        }
    }
    // Cargo already fingerprints compiler identity. Explicit binary/runtime
    // files cover replacement of an installation without changing its path.
    files.insert(std::env::current_exe().map_err(|e| e.to_string())?);
    let (runtime, tcc) = crate::tcc::tcc_paths();
    files.insert(tcc);
    add_runtime_files(&runtime, &mut files)?;
    let names: Vec<String> = files.iter().map(|p| escape_path(p)).collect::<Result<_,_>>()?;
    let mut text = format!("{}: {}\n\n", escape_path(output)?, names.join(" "));
    for name in names { text.push_str(&format!("{name}:\n")); }
    for (name, value) in envs {
        text.push_str("# env-dep:");
        text.push_str(&escape_env(&name));
        if let Some(value) = value { text.push('='); text.push_str(&escape_env(&value)); }
        text.push('\n');
    }
    std::fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn add_runtime_files(root: &Path, files: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|e| format!("native runtime {}: {e}", root.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() { add_runtime_files(&path, files)?; }
        else if path.is_file() { files.insert(path); }
    }
    Ok(())
}

fn diagnostics(text: &str, json: bool) {
    if text.is_empty() { return; }
    if json {
        let rendered = crate::diag::json_str(text);
        eprintln!("{{\"$message_type\":\"diagnostic\",\"message\":{rendered},\"code\":null,\"level\":\"error\",\"spans\":[],\"children\":[],\"rendered\":{rendered}}}");
    } else { eprint!("{text}"); }
}

fn compile(args: &Args) -> Result<i32, String> {
    let native_env: BTreeMap<String, Option<String>> = NATIVE_ENV_INPUTS.iter()
        .map(|&name| (name.to_string(), std::env::var(name).ok())).collect();
    // Capture the calling environment before choosing internal cache storage.
    // env!/option_env! must never observe our implementation's own settings.
    let mut original_env = Vec::new();
    for (key, value) in std::env::vars_os() {
        // A non-UTF-8 name cannot be named by a Rust env! string. A valid
        // name with a non-UTF-8 value must not silently turn into None.
        if let Ok(key) = key.into_string() {
            let value = value.into_string().map_err(|_| format!(
                "environment variable `{key}` has a non-UTF-8 value; this native entry point requires UTF-8 environment values"))?;
            original_env.push((key, value));
        }
    }
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(|n| n.starts_with("LUST_") && !matches!(n,
            "LUST_CACHE_DIR" | "LUST_NO_CACHE" | "LUST_NO_DAEMON" | "LUST_CACHE_MAX_BYTES")) {
            return Err(format!("native configuration override `{}` is not qualified for the Cargo entry point", name.to_string_lossy()));
        }
    }
    if !args.types.is_empty() && args.types != ["bin"] { return Err("native Cargo entry point currently emits only bin artifacts; rlib, metadata and proc macros are not implemented".into()); }
    if args.edition.as_deref() != Some("2024") { return Err("native Cargo entry point currently requires --edition=2024".into()); }
    if args.warnings { return Err("native lint configuration is not implemented".into()); }
    if args.strip_debug && args.debug { return Err("stripping an artifact built with native debug information is not implemented".into()); }
    for cfg in &args.check_cfg {
        if cfg != "cfg(docsrs,test)" && !feature_check_cfg(cfg) {
            return Err(format!("custom check-cfg `{cfg}` is not implemented"));
        }
    }
    if args.emits.iter().any(|v| v != "link" && v != "dep-info") {
        return Err("native Cargo entry point supports --emit=link,dep-info only; Rust metadata is not implemented".into());
    }
    if !args.emits.is_empty() && !args.emits.iter().any(|v| v == "link") { return Err("native dep-info without link is not implemented".into()); }
    if args.output.is_some() && args.emits.iter().any(|v| v == "dep-info") { return Err("-o with dep-info is not implemented; use --out-dir".into()); }
    let typed = args.source.as_ref().ok_or("missing Rust source file")?;
    if typed == Path::new("-") { return Err("native source from stdin is not implemented".into()); }
    let source = typed.canonicalize().map_err(|e| format!("input {}: {e}", typed.display()))?;
    let name = args.name.clone().unwrap_or_else(|| typed.file_stem().unwrap_or_default().to_string_lossy().into_owned());
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') { return Err("invalid crate name".into()); }
    CRATE_INPUT.set((source.clone(), name.clone())).map_err(|_| "one native crate per process is required")?;
    let output = args.output.clone().unwrap_or_else(|| args.out_dir.clone().unwrap_or_else(|| PathBuf::from(".")).join(format!("{name}{}{}", args.extra, std::env::consts::EXE_SUFFIX)));
    let output = std::path::absolute(output).map_err(|e| e.to_string())?;
    let dep = args.out_dir.clone().unwrap_or_else(|| output.parent().unwrap().into()).join(format!("{name}{}.d", args.extra));
    let mut key = SessionKey::synthetic(&source, false);
    key.krate.target_name = name;
    key.krate.package_id = format!("cargo-native:{}:{}", source.display(), args.metadata);
    key.profile = profile(args);
    key.features = args.features.clone();
    key.target_triple = args.target.clone().unwrap_or_default();
    // Cargo's -Cincremental directory belongs to this compiler invocation.
    // Reuse Lust's genuine unit-cache implementation under it, never rustc's
    // on-disk format or a whole-request result that omits external inputs.
    if let Some(path) = &args.incremental {
        if std::env::var_os("LUST_CACHE_DIR").is_some() { return Err("-Cincremental conflicts with LUST_CACHE_DIR".into()); }
        // SAFETY: single-threaded compiler startup, before native workers exist.
        unsafe { std::env::set_var("LUST_CACHE_DIR", path.join("lustr-native")); }
    }
    crate::tcc::use_mimalloc();
    crate::tcc::set_huge_pages(false);
    raise_stack_limit();
    let object_dir = temporary_directory()?;
    let mut session = crate::driver::Session::new(object_dir.clone(), false, key.clone());
    session.caches.leak_at_exit = true;
    let request = Request {
        cmd: Cmd::Build, session: key, file: source.clone(), out: output.clone(),
        json: false, timings: false, debug: args.debug, emit_c: None, max_errors: 0,
        run_args: None, harness: false, cc: false,
        client: Client { env: Some(original_env), shown: Some(typed.clone()) },
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::driver::compile(&request, &mut session)));
    let response = match result {
        Ok(response) => response,
        Err(panic) => { let _ = std::fs::remove_dir_all(&object_dir); std::panic::resume_unwind(panic) }
    };
    diagnostics(&response.stderr, args.json);
    if !response.stdout.is_empty() { print!("{}", response.stdout); }
    let dep_result: Result<(), String> = if response.code == 0 {
        (|| {
            if args.strip_debug { verify_no_debug_sections(&output)?; }
            if args.emits.iter().any(|v| v == "dep-info") {
                dependency_info(&session, &source, &output, &dep, native_env)?;
            }
            Ok(())
        })()
    } else { Ok(()) };
    let _ = std::fs::remove_dir_all(&object_dir);
    std::mem::forget(session);
    if let Err(error) = dep_result {
        // A successful native link followed by a failed artifact contract is
        // a compiler failure. Do not leave the just-produced unqualified file.
        let _ = std::fs::remove_file(&output);
        if args.emits.iter().any(|v| v == "dep-info") { let _ = std::fs::remove_file(&dep); }
        return Err(error);
    }
    if response.code == 0 && args.artifacts {
        eprintln!("{{\"$message_type\":\"artifact\",\"artifact\":{},\"emit\":\"link\"}}", crate::diag::json_str(output.to_str().ok_or("non-UTF-8 output path")?));
    }
    Ok(response.code)
}

/// TinyCC's non-debug output normally already satisfies Cargo's strip request.
/// Verify the actual ELF rather than ignoring that option or invoking another
/// process. Other formats and any remaining debug sections are explicit errors.
fn verify_no_debug_sections(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("inspect native ELF: {e}"))?;
    let fail = || "-Cstrip=debuginfo requires a verifiable ELF64 little-endian native artifact without debug sections".to_string();
    if bytes.get(..6) != Some(b"\x7fELF\x02\x01") { return Err(fail()); }
    let u16_at = |at: usize| bytes.get(at..at + 2).map(|v| u16::from_le_bytes(v.try_into().unwrap()));
    let u32_at = |at: usize| bytes.get(at..at + 4).map(|v| u32::from_le_bytes(v.try_into().unwrap()));
    let u64_at = |at: usize| bytes.get(at..at + 8).map(|v| u64::from_le_bytes(v.try_into().unwrap()));
    let table = usize::try_from(u64_at(40).ok_or_else(fail)?).map_err(|_| fail())?;
    let size = usize::from(u16_at(58).ok_or_else(fail)?);
    let count = usize::from(u16_at(60).ok_or_else(fail)?);
    let strings_index = usize::from(u16_at(62).ok_or_else(fail)?);
    if size != 64 || count == 0 || strings_index == 0 || strings_index >= count { return Err(fail()); }
    let table_size = size.checked_mul(count).ok_or_else(fail)?;
    let table_end = table.checked_add(table_size).ok_or_else(fail)?;
    if table_end > bytes.len() { return Err(fail()); }
    let strings_header = table + strings_index * size;
    let strings_start = usize::try_from(u64_at(strings_header + 24).ok_or_else(fail)?).map_err(|_| fail())?;
    let strings_size = usize::try_from(u64_at(strings_header + 32).ok_or_else(fail)?).map_err(|_| fail())?;
    let strings_end = strings_start.checked_add(strings_size).ok_or_else(fail)?;
    let strings = bytes.get(strings_start..strings_end).ok_or_else(fail)?;
    for index in 0..count {
        let name_at = u32_at(table + index * size).ok_or_else(fail)? as usize;
        let tail = strings.get(name_at..).ok_or_else(fail)?;
        let end = tail.iter().position(|&b| b == 0).ok_or_else(fail)?;
        let name = &tail[..end];
        if [b".debug".as_slice(), b".zdebug", b".stab", b".gnu_debug", b".gdb_index", b".BTF"]
            .iter().any(|prefix| name.starts_with(prefix)) {
            return Err(format!("native artifact still contains debug section {}; stripping is not implemented", String::from_utf8_lossy(name)));
        }
    }
    Ok(())
}

fn feature_check_cfg(value: &str) -> bool {
    let Some(body) = value.strip_prefix("cfg(feature, values(").and_then(|s| s.strip_suffix("))")) else { return false; };
    let body = body.trim();
    if body.is_empty() { return true; }
    let body = body.strip_suffix(',').unwrap_or(body).trim();
    if body.is_empty() { return false; }
    body.split(',').all(|item| {
        item.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"'))
            .is_some_and(|s| !s.contains(['"','\\','\n','\r']))
    })
}

fn temporary_directory() -> Result<PathBuf, String> {
    let start = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    for attempt in 0..100u32 {
        let dir = std::env::temp_dir().join(format!("lustr-native-{}-{start}-{attempt}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(e) => return Err(format!("temporary native build directory: {e}")),
        }
    }
    Err("could not allocate a fresh native build directory".into())
}

// The native parser accepts expressions whose recursion exceeds the OS default
// main stack. Match the native entry point's startup allowance without a second
// compiler process or a compile thread. Linux is the qualified package host.
#[cfg(target_os = "linux")]
fn raise_stack_limit() {
    #[repr(C)]
    struct Limit { current: u64, maximum: u64 }
    unsafe extern "C" {
        fn getrlimit(resource: i32, limit: *mut Limit) -> i32;
        fn setrlimit(resource: i32, limit: *const Limit) -> i32;
    }
    let mut limit = Limit { current: 0, maximum: 0 };
    // SAFETY: on the qualified 64-bit Linux target, Limit has the libc rlimit
    // layout. The new soft limit never exceeds the existing hard limit.
    unsafe {
        if getrlimit(3, &mut limit) == 0 {
            limit.current = limit.current.max((crate::pool::STACK_SIZE as u64).min(limit.maximum));
            let _ = setrlimit(3, &limit);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn raise_stack_limit() {}

pub fn run(raw: Vec<OsString>) -> i32 {
    let args = match parse(raw) { Ok(args) => args, Err(e) => { eprintln!("error: lustr-native: {e}"); return 2; } };
    if args.version { show_version(args.verbose); return 0; }
    if let Some(target) = &args.target {
        if target != crate::protocol::host_triple() { diagnostics(&format!("error: native Cargo entry point does not support target `{target}`\n"), args.json); return 2; }
    }
    let answer = if !args.prints.is_empty() { print_info(&args).map(|_| 0) } else { compile(&args) };
    match answer {
        Ok(code) => code,
        Err(e) => { diagnostics(&format!("error: lustr-native: {e}\n"), args.json); 2 }
    }
}
