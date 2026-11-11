#![forbid(unsafe_code)]
//! Cargo owns the dependency graph and all Rust artefacts. This module chooses an
//! entire invocation, never pieces of source or incompatible library formats.
mod doctor;
mod toolchain;

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;
use std::process::Command;

const HELP: &str = "lustr compiler set 0.1.0
Coordinate Lust subset compilation and full Rust projects.

Usage:
  lustr [+toolchain] [--engine ENGINE] <command> [arguments...]
  lustr --bluefe-root /absolute/sysroot <command> [arguments...]
  lustr --bluefe-root /absolute/sysroot --frontend-jobs N rustc [Cargo arguments] [-- rustc arguments]
  cargo lustr [--engine ENGINE] <command> [arguments...]
  cargo lust [--engine auto|rustc|cranelift|lust] <command> [arguments...]
  cargo-lust [+toolchain] [--engine ENGINE] <command> [arguments...]
  cargo lust [--engine ENGINE] plan <command> [arguments...]
  cargo lust [--engine ENGINE] --explain <command> [arguments...]
  cargo lust doctor

Engines:
  auto       Preserve the project's selected Cargo/rustc configuration (default).
  rustc      Invoke Cargo unchanged, preserving its selected compiler and wrappers.
  cranelift  Experimental Cargo dev/test code generation with Cranelift. Requires
             LUST_CRANELIFT_BACKEND set to an existing absolute backend dylib, built
             with unwinding support for the selected nightly rustc. The stock
             rustup preview backend is not selected: it loses panic semantics.
             Release/custom profiles and commands other than build/run/test stay
             on the project's original Cargo configuration.
  lust       Forward native Lust commands and arguments to Lust for a complete
             supported target. For example: --engine lust check src/main.rs.

Coordinator options must precede the command. All arguments after it are passed
through unchanged, including arguments after --. Plan/--explain prints JSON and
runs no compiler, Cargo command, build script or project program.

--frontend-jobs is an opt-in experiment for one Cargo rustc target. It appends
-Zunstable-options and --jobs-frontend=N to that target's compiler arguments.
Use --profile check for metadata checking. Cargo still selects project flags,
wrappers, features and dependencies. Other commands are rejected in this mode;
no workspace-wide Rust flag or wrapper configuration is replaced.

No toolchains are installed automatically. A relocatable package selects its
bundled Cargo/rustc when no explicit toolchain or tool override is supplied.
An ordinary coordinator installation preserves the existing tool selection.
No compilation or program is retried after failure. The coordinator does not change panic, features, target,
optimization, source, dependency versions, configuration files or Rust flags
except the explicitly requested frontend job options above.
Cranelift selection uses command-scoped Cargo --config arguments, so nested Cargo
processes do not inherit backend profile environment variables from this tool.
Auto currently uses Cargo/rustc because arbitrary Lust/Cargo interoperability has
not been established. It does not claim a speed increase.

LUST_CARGO selects the Cargo executable. LUSTC selects the Lust executable.
Otherwise CARGO selects Cargo, then PATH. Lust is found beside cargo-lust, then on PATH.
An explicit +toolchain runs `rustup run -- TOOLCHAIN cargo` without installing it.
This discards inherited CARGO and cannot be combined with LUST_CARGO.
LUST_CRANELIFT_BACKEND is required only when the Cranelift dev/test route is used.
Its path must be valid UTF-8 for Cargo's command-scoped TOML configuration.
Choosing a backend path does not certify its features, ABI, or program correctness.
The lustr, cargo-lustr, and cargo-lust executables use the same coordinator.
--bluefe-root selects bin/cargo, bin/rustc and bin/rustdoc for this invocation.
It preserves arguments, flags, profiles and compiler wrappers, installs nothing,
and cannot be combined with +toolchain or --engine lust. Plans run no probes;
doctor reports availability rather than semantic or backend qualification.
Packaged discovery requires lustr-toolchain.json beside bin and selects paths
relative to the running executable. This marker identifies a package; it does not
certify native Rust coverage or performance. +toolchain, --bluefe-root, LUST_CARGO,
RUSTC, RUSTDOC, RUSTUP_TOOLCHAIN, and a different CARGO preserve explicit selection.
Native Lust uses its existing sibling executable and runtime discovery unchanged.
";

/// Shared command entry point for the direct and both Cargo extension names.
/// Cargo adds its subcommand name to argv; remove only that leading name.
pub fn run_cli(args: &[OsString]) -> i32 {
    let args = if args.first().is_some_and(|arg| arg == "lust" || arg == "lustr") { &args[1..] } else { args };
    run(args)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Engine {
    Auto,
    Rustc,
    Cranelift,
    Lust,
}

impl Engine {
    fn parse(value: &OsStr) -> Result<Self, String> {
        match value.to_str() {
            Some("auto") => Ok(Self::Auto),
            Some("rustc") => Ok(Self::Rustc),
            Some("cranelift") => Ok(Self::Cranelift),
            Some("lust") => Ok(Self::Lust),
            _ => Err(format!("unknown engine {value:?}; expected auto, rustc, cranelift or lust")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rustc => "rustc",
            Self::Cranelift => "cranelift",
            Self::Lust => "lust",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Request {
    pub engine: Engine,
    pub explain: bool,
    pub toolchain: Option<OsString>,
    pub bluefe_root: Option<PathBuf>,
    pub frontend_jobs: Option<usize>,
    pub args: Vec<OsString>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Action {
    Help,
    Version,
    Execute(Request),
}

pub fn parse(args: &[OsString]) -> Result<Action, String> {
    if args.is_empty() {
        return Ok(Action::Help);
    }
    let mut engine = Engine::Auto;
    let mut explain = false;
    let mut toolchain = None;
    let mut bluefe_root = None;
    let mut frontend_jobs = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--help" || arg == "-h" {
            return Ok(Action::Help);
        }
        if arg == "--version" || arg == "-V" {
            return Ok(Action::Version);
        }
        if arg == "--explain" || arg == "plan" {
            explain = true;
        } else if arg == "--engine" {
            index += 1;
            engine = Engine::parse(args.get(index).ok_or("--engine requires a value")?)?;
        } else if let Some(value) = arg.to_str().and_then(|s| s.strip_prefix("--engine=")) {
            engine = Engine::parse(OsStr::new(value))?;
        } else if arg == "--frontend-jobs" || arg.to_str().is_some_and(|s| s.starts_with("--frontend-jobs=")) {
            let value = if arg == "--frontend-jobs" {
                index += 1;
                args.get(index).and_then(|value| value.to_str())
            } else {
                arg.to_str().and_then(|value| value.strip_prefix("--frontend-jobs="))
            };
            let jobs = value
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|jobs| *jobs > 0)
                .ok_or("--frontend-jobs requires a positive integer")?;
            if frontend_jobs.replace(jobs).is_some() {
                return Err("specify --frontend-jobs only once".to_owned());
            }
        } else if arg == "--bluefe-root" {
            index += 1;
            let value = args.get(index).filter(|value| !value.is_empty()).ok_or("--bluefe-root requires a sysroot directory")?;
            if bluefe_root.replace(PathBuf::from(value)).is_some() {
                return Err("specify --bluefe-root only once".to_owned());
            }
        } else if arg.to_str().is_some_and(|s| s.starts_with('+')) {
            if arg == "+" {
                return Err("+toolchain requires a toolchain name".to_owned());
            }
            if toolchain.replace(arg.clone()).is_some() {
                return Err("specify only one +toolchain".to_owned());
            }
        } else if arg == "--" {
            index += 1;
            break;
        } else {
            break;
        }
        index += 1;
    }
    if index == args.len() {
        return Err("a command is required; run cargo lust --help".to_owned());
    }
    if engine == Engine::Lust && toolchain.is_some() {
        return Err("+toolchain selects Cargo, not the Lust compiler; omit it for --engine lust".to_owned());
    }
    if bluefe_root.is_some() && (toolchain.is_some() || engine == Engine::Lust) {
        return Err("--bluefe-root selects a full Rust toolchain; do not combine it with +toolchain or --engine lust".to_owned());
    }
    if bluefe_root.as_ref().is_some_and(|root| !root.is_absolute()) {
        return Err("--bluefe-root must be an absolute sysroot path".to_owned());
    }
    let request = Request { engine, explain, toolchain, bluefe_root, frontend_jobs, args: args[index..].to_vec() };
    validate_frontend_request(&request)?;
    Ok(Action::Execute(request))
}

fn validate_frontend_request(request: &Request) -> Result<(), String> {
    if let Some(jobs) = request.frontend_jobs {
        if jobs == 0
            || request.bluefe_root.is_none()
            || !matches!(request.engine, Engine::Auto | Engine::Rustc)
            || request.args.first().is_none_or(|arg| arg != "rustc")
        {
            return Err("--frontend-jobs requires --bluefe-root and the Cargo rustc command with engine auto or rustc".to_owned());
        }
        let compiler_args: Vec<_> = request.args.iter().skip_while(|arg| *arg != "--").skip(1).collect();
        if compiler_args.iter().enumerate().any(|(index, arg)| {
            arg.to_str().is_some_and(|arg| {
                arg == "--jobs-frontend"
                    || arg.starts_with("--jobs-frontend=")
                    || arg == "-Zthreads"
                    || arg.starts_with("-Zthreads=")
                    || (arg == "-Z"
                        && compiler_args
                            .get(index + 1)
                            .and_then(|next| next.to_str())
                            .is_some_and(|next| next == "threads" || next.starts_with("threads=")))
            })
        }) {
            return Err("frontend job options already occur in the explicit rustc arguments; select them in one place".to_owned());
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Programs {
    pub cargo: OsString,
    pub lust: OsString,
    pub explicit_cargo: bool,
    pub cranelift_backend: Option<OsString>,
}

impl Programs {
    pub fn from_environment() -> Self {
        let explicit_cargo = std::env::var_os("LUST_CARGO");
        let cargo = explicit_cargo.clone().or_else(|| std::env::var_os("CARGO")).unwrap_or_else(|| OsString::from("cargo"));
        let lust = std::env::var_os("LUSTC")
            .unwrap_or_else(|| sibling_lust().map(PathBuf::into_os_string).unwrap_or_else(|| OsString::from("lust")));
        Self { cargo, lust, explicit_cargo: explicit_cargo.is_some(), cranelift_backend: std::env::var_os("LUST_CRANELIFT_BACKEND") }
    }
}

fn sibling_lust() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let name = if cfg!(windows) { "lust.exe" } else { "lust" };
    let path = exe.parent()?.join(name);
    path.is_file().then_some(path)
}

#[derive(Debug, Eq, PartialEq)]
pub struct Plan {
    pub requested_engine: Engine,
    pub selected_engine: Engine,
    pub reason: &'static str,
    pub program: OsString,
    pub args: Vec<OsString>,
    pub environment: Vec<(OsString, OsString)>,
    pub environment_remove: Vec<OsString>,
    pub codegen_backend: Option<OsString>,
    pub experimental: bool,
}

/// Construct a complete invocation without reading a project or running tools.
/// This is deliberately conservative: only known dev/test commands receive a
/// different backend. Cargo itself resolves graph, configuration and profiles.
pub fn plan(request: &Request, mut programs: Programs) -> Result<Plan, String> {
    validate_frontend_request(request)?;
    if request.bluefe_root.is_some() && (request.toolchain.is_some() || request.engine == Engine::Lust) {
        return Err("--bluefe-root cannot be combined with +toolchain or --engine lust".to_owned());
    }
    let environment = request.bluefe_root.as_deref().map(toolchain::environment).unwrap_or_default();
    if let Some(root) = &request.bluefe_root {
        programs.cargo = toolchain::program(root, "cargo").into_os_string();
    }
    if request.engine == Engine::Lust {
        if programs.lust.is_empty() {
            return Err("LUSTC must be a nonempty executable path when set".to_owned());
        }
        return Ok(Plan {
            requested_engine: request.engine,
            selected_engine: Engine::Lust,
            reason: "explicit Lust target; no Rust library artefacts are exchanged with Cargo",
            program: programs.lust,
            args: request.args.clone(),
            environment: Vec::new(),
            environment_remove: Vec::new(),
            codegen_backend: None,
            experimental: true,
        });
    }
    let mut args = Vec::new();
    let mut environment_remove = Vec::new();
    let program = if let Some(toolchain) = &request.toolchain {
        if programs.explicit_cargo {
            return Err("+toolchain cannot be combined with LUST_CARGO; select either a rustup toolchain or a Cargo executable".to_owned());
        }
        // CARGO inherited from `cargo lust` points at that invocation's actual
        // Cargo binary, not necessarily a rustup proxy. It cannot parse +name.
        // `run` without --install requires an already installed toolchain. The
        // option terminator prevents a name such as +--install becoming a flag.
        let name =
            toolchain.to_str().and_then(|value| value.strip_prefix('+')).filter(|value| !value.is_empty()).ok_or("invalid +toolchain")?;
        args.extend([OsString::from("run"), OsString::from("--"), OsString::from(name), OsString::from("cargo")]);
        environment_remove.push(OsString::from("CARGO"));
        OsString::from("rustup")
    } else {
        if programs.cargo.is_empty() {
            return Err("LUST_CARGO or CARGO must be a nonempty executable path when set".to_owned());
        }
        programs.cargo
    };
    let mut selected_engine = Engine::Rustc;
    let mut codegen_backend = None;
    let mut forwarded_args = request.args.as_slice();
    let reason = match request.engine {
        Engine::Auto => {
            "full project compatibility uses the selected Cargo/rustc configuration; automatic Lust routing is not yet qualified"
        }
        Engine::Rustc => "explicit Cargo/rustc; preserve compiler, wrappers, profiles and arguments",
        Engine::Cranelift => {
            if !is_codegen_command(&request.args) {
                "command does not request the Cranelift development build path; preserve Cargo configuration"
            } else if has_profile_selection(&request.args) {
                "explicit release/profile selection; preserve Cargo configuration"
            } else {
                let backend = configured_cranelift_backend(programs.cranelift_backend.as_deref())?;
                selected_engine = Engine::Cranelift;
                // Keep repeated options in the subcommand's argument scope.
                // A caller's --config after `build` replaces the entire list
                // supplied before `build`, rather than extending that list.
                args.push(request.args[0].clone());
                forwarded_args = &request.args[1..];
                args.push(OsString::from("-Zcodegen-backend"));
                // Profile environment variables would also reach applications,
                // tests and build scripts. Their ordinary nested Cargo commands
                // would then require -Zcodegen-backend unexpectedly. Scope these
                // settings to this Cargo invocation instead.
                let value = toml_string(backend);
                for profile in ["dev", "test"] {
                    args.push(OsString::from("--config"));
                    args.push(OsString::from(format!("profile.{profile}.codegen-backend={value}")));
                }
                codegen_backend = Some(OsString::from(backend));
                "experimental configured Cranelift dev/test backend; no panic or feature flags are changed, and backend correctness requires qualification"
            }
        }
        Engine::Lust => unreachable!(),
    };
    args.extend(forwarded_args.iter().cloned());
    let reason = if let Some(jobs) = request.frontend_jobs {
        if !request.args.iter().any(|arg| arg == "--") {
            args.push("--".into());
        }
        args.extend(["-Zunstable-options".into(), format!("--jobs-frontend={jobs}").into()]);
        "experimental explicit frontend scheduling for one Cargo rustc target; all caller arguments, project settings and compiler wrappers are preserved"
    } else {
        reason
    };
    Ok(Plan {
        requested_engine: request.engine,
        selected_engine,
        reason,
        program,
        args,
        environment,
        environment_remove,
        codegen_backend,
        experimental: selected_engine == Engine::Cranelift || request.frontend_jobs.is_some(),
    })
}

fn configured_cranelift_backend(backend: Option<&OsStr>) -> Result<&str, String> {
    let Some(backend) = backend.filter(|value| !value.is_empty()) else {
        return Err("Cranelift dev/test builds require LUST_CRANELIFT_BACKEND=/absolute/path/to/backend-dylib built with unwinding support for the selected nightly; the stock rustup preview backend is not selected because it loses panic semantics".to_owned());
    };
    let backend = backend.to_str().ok_or("LUST_CRANELIFT_BACKEND must be valid UTF-8 for Cargo's command-scoped TOML configuration")?;
    let path = std::path::Path::new(backend);
    if !path.is_absolute() {
        return Err("LUST_CRANELIFT_BACKEND must be an absolute backend dylib path; named backends such as cranelift and relative paths are not accepted".to_owned());
    }
    if !path.is_file() {
        return Err(format!("LUST_CRANELIFT_BACKEND does not name an existing backend file: {backend:?}"));
    }
    // Cargo and rustc validate the library and compiler ABI. Do not load third-party
    // code as a preflight, and do not claim a path proves its build features.
    Ok(backend)
}

/// One TOML basic string, passed as an argument directly to Cargo. TOML forbids
/// raw control characters, including DEL, and backslashes must remain literal
/// path bytes rather than becoming escape sequences such as a Windows `\t`.
fn toml_string(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{8}' => result.push_str("\\b"),
            '\t' => result.push_str("\\t"),
            '\n' => result.push_str("\\n"),
            '\u{c}' => result.push_str("\\f"),
            '\r' => result.push_str("\\r"),
            ch if ch.is_control() && u32::from(ch) <= 0xffff => result.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch if ch.is_control() => result.push_str(&format!("\\U{:08x}", u32::from(ch))),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

fn is_codegen_command(args: &[OsString]) -> bool {
    // Even Cargo's familiar b/r/t shortcuts can be overridden by project
    // aliases, which may select a release profile or a different command.
    // Cargo, not this coordinator, must resolve those aliases.
    matches!(args.first().and_then(|a| a.to_str()), Some("build" | "run" | "test"))
}

fn has_profile_selection(args: &[OsString]) -> bool {
    args.iter().skip(1).take_while(|arg| *arg != "--").any(|arg| {
        arg == "--release" || arg == "--profile" || arg.to_str().is_some_and(|s| s.starts_with("--profile=") || short_release_flag(s))
    })
}

fn short_release_flag(arg: &str) -> bool {
    let Some(flags) = arg.strip_prefix('-').filter(|flags| !flags.starts_with('-')) else {
        return false;
    };
    // Cargo accepts groups such as -vr. Stop at a value-taking or unknown flag
    // so that -pserver, -Ffeature, and -Zflag never look like --release.
    for flag in flags.chars() {
        match flag {
            'r' => return true,
            'v' | 'q' | 'h' => {}
            _ => return false,
        }
    }
    false
}

impl Plan {
    pub fn json(&self) -> String {
        let args = self.args.iter().map(|s| os_json(s)).collect::<Vec<_>>().join(",");
        let env = self.environment.iter().map(|(key, value)| format!("{}:{}", os_json(key), os_json(value))).collect::<Vec<_>>().join(",");
        let removed = self.environment_remove.iter().map(|key| os_json(key)).collect::<Vec<_>>().join(",");
        let backend = self.codegen_backend.as_deref().map_or_else(|| "null".to_owned(), os_json);
        format!(
            "{{\"format_version\":1,\"requested_engine\":{},\"selected_engine\":{},\"reason\":{},\"experimental\":{},\"program\":{},\"args\":[{}],\"environment\":{{{}}},\"environment_remove\":[{}],\"codegen_backend\":{}}}",
            string_json(self.requested_engine.name()),
            string_json(self.selected_engine.name()),
            string_json(self.reason),
            self.experimental,
            os_json(&self.program),
            args,
            env,
            removed,
            backend,
        )
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        for key in &self.environment_remove {
            command.env_remove(key);
        }
        command.args(&self.args).envs(self.environment.iter().map(|(key, value)| (key, value)));
        command
    }
}

// JSON strings never lose OS argument bytes. On Unix invalid UTF-8 is an
// explicit {"unix_bytes_hex":"..."} object; Windows uses UTF-16 code units.
fn os_json(value: &OsStr) -> String {
    if let Some(value) = value.to_str() {
        return string_json(value);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let bytes = value.as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
        format!("{{\"unix_bytes_hex\":\"{bytes}\"}}")
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let units = value.encode_wide().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
        format!("{{\"windows_utf16\":[{units}]}}")
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Rust's encoded OS bytes are lossless within this platform/version.
        let bytes = value.as_encoded_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
        format!("{{\"os_bytes_hex\":\"{bytes}\"}}")
    }
}

fn string_json(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch < '\u{20}' => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

pub fn run(args: &[OsString]) -> i32 {
    let mut request = match parse(args) {
        Ok(Action::Help) => {
            print!("{HELP}");
            return 0;
        }
        Ok(Action::Version) => {
            println!("lustr {}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
        Ok(Action::Execute(request)) => request,
        Err(error) => {
            eprintln!("lustr: {error}");
            return 2;
        }
    };
    if request.bluefe_root.is_none() && request.toolchain.is_none() && request.engine != Engine::Lust {
        request.bluefe_root = toolchain::packaged_root();
    }
    if !request.explain {
        if let Some(root) = &request.bluefe_root {
            if let Err(error) = toolchain::validate(root) {
                eprintln!("lustr: {error}");
                return 2;
            }
        }
        if request.args == [OsString::from("doctor")] {
            return doctor::run_for_root(request.bluefe_root.as_deref());
        }
    }
    let plan = match plan(&request, Programs::from_environment()) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("lustr: {error}");
            return 2;
        }
    };
    if request.explain {
        println!("{}", plan.json());
        return 0;
    }
    if plan.selected_engine == Engine::Cranelift {
        eprintln!(
            "lustr: experimental configured Cranelift backend; use a backend built with unwinding support for this nightly. Panic and feature flags are unchanged. The selected backend still requires semantic qualification; failures are returned without retry."
        );
    }
    execute(&plan)
}

#[cfg(unix)]
fn execute(plan: &Plan) -> i32 {
    use std::os::unix::process::CommandExt;
    let error = plan.command().exec();
    spawn_error(&plan.program, error)
}

#[cfg(not(unix))]
fn execute(plan: &Plan) -> i32 {
    match plan.command().status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => spawn_error(&plan.program, error),
    }
}

fn spawn_error(program: &OsStr, error: io::Error) -> i32 {
    eprintln!("lustr: cannot execute {program:?}: {error}");
    if error.kind() == io::ErrorKind::NotFound { 127 } else { 126 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(input: &[&str]) -> Vec<OsString> {
        input.iter().map(OsString::from).collect()
    }
    fn request(input: &[&str]) -> Request {
        match parse(&words(input)).unwrap() {
            Action::Execute(r) => r,
            other => panic!("{other:?}"),
        }
    }
    fn mock_backend() -> OsString {
        // A real file stands in for a configured library. Planning must never
        // try to dynamically load it or claim it was qualified as a backend.
        std::env::current_exe().unwrap().into_os_string()
    }
    fn prepared(input: &[&str]) -> Plan {
        plan(
            &request(input),
            Programs {
                cargo: OsString::from("/tools with spaces/cargo"),
                lust: OsString::from("/tools/lust"),
                explicit_cargo: false,
                cranelift_backend: Some(mock_backend()),
            },
        )
        .unwrap()
    }

    fn backend_config_args() -> Vec<OsString> {
        let backend = mock_backend();
        let quoted = toml_string(backend.to_str().unwrap());
        vec![
            "-Zcodegen-backend".into(),
            "--config".into(),
            format!("profile.dev.codegen-backend={quoted}").into(),
            "--config".into(),
            format!("profile.test.codegen-backend={quoted}").into(),
        ]
    }

    #[test]
    fn auto_is_cargo_compatibility_without_speculation() {
        let plan = prepared(&["build", "--workspace", "--locked"]);
        assert_eq!(plan.requested_engine, Engine::Auto);
        assert_eq!(plan.selected_engine, Engine::Rustc);
        assert_eq!(plan.args, words(&["build", "--workspace", "--locked"]));
        assert!(plan.environment.is_empty());
        assert!(plan.codegen_backend.is_none());
        assert!(plan.json().contains("\"codegen_backend\":null"));
    }

    #[test]
    fn coordinator_options_stop_at_command() {
        let req = request(&["--engine", "rustc", "run", "--", "--engine", "lust", "--explain"]);
        assert_eq!(req.engine, Engine::Rustc);
        assert!(!req.explain);
        assert_eq!(req.args, words(&["run", "--", "--engine", "lust", "--explain"]));
    }

    #[test]
    fn plan_and_explain_only_change_execution_intent() {
        let req = request(&["--engine=cranelift", "plan", "build"]);
        assert!(req.explain);
        assert_eq!(req.engine, Engine::Cranelift);
        assert_eq!(req.args, words(&["build"]));
        assert!(request(&["--explain", "check"]).explain);
    }

    #[test]
    fn preserves_explicit_toolchain_selection() {
        let plan = prepared(&["+nightly-2026-11-07", "--engine=cranelift", "build", "--offline"]);
        assert_eq!(plan.program, OsStr::new("rustup"));
        let mut expected = words(&["run", "--", "nightly-2026-11-07", "cargo", "build"]);
        expected.extend(backend_config_args());
        expected.extend(words(&["--offline"]));
        assert_eq!(plan.args, expected);
        assert!(plan.environment.is_empty());
        assert_eq!(plan.environment_remove, words(&["CARGO"]));
        assert_eq!(plan.codegen_backend, Some(mock_backend()));
    }

    #[test]
    fn toolchain_name_cannot_enable_rustup_install() {
        let plan = prepared(&["+--install", "build"]);
        assert_eq!(plan.args, words(&["run", "--", "--install", "cargo", "build"]));
    }

    #[test]
    fn explicit_cargo_and_toolchain_cannot_conflict() {
        let result = plan(
            &request(&["+nightly", "build"]),
            Programs {
                cargo: OsString::from("/custom/cargo"),
                lust: OsString::from("lust"),
                explicit_cargo: true,
                cranelift_backend: None,
            },
        );
        assert!(result.unwrap_err().contains("LUST_CARGO"));
    }

    #[test]
    fn irrelevant_empty_programs_do_not_block_the_selected_engine() {
        let cargo = plan(
            &request(&["build"]),
            Programs { cargo: OsString::from("cargo"), lust: OsString::new(), explicit_cargo: false, cranelift_backend: None },
        );
        assert!(cargo.is_ok());
        let lust = plan(
            &request(&["--engine=lust", "check", "a.rs"]),
            Programs { cargo: OsString::new(), lust: OsString::from("lust"), explicit_cargo: true, cranelift_backend: None },
        );
        assert!(lust.is_ok());
    }

    #[test]
    fn dev_codegen_commands_scope_backend_configuration_to_cargo_arguments() {
        for command in ["build", "run", "test"] {
            let plan = prepared(&["--engine", "cranelift", command, "--features", "a,b"]);
            assert_eq!(plan.selected_engine, Engine::Cranelift);
            let mut expected = words(&[command]);
            expected.extend(backend_config_args());
            expected.extend(words(&["--features", "a,b"]));
            assert_eq!(plan.args, expected);
            assert!(plan.environment.is_empty());
            assert!(plan.environment_remove.is_empty());
            assert_eq!(plan.codegen_backend, Some(mock_backend()));
            assert!(plan.json().contains(&format!("\"codegen_backend\":{}", os_json(&mock_backend()))));
        }
    }

    #[test]
    fn backend_toml_strings_escape_path_metacharacters_and_controls() {
        let value = "C:\\tools\\\"backend\"\u{8}\t\n\u{c}\r\u{1}\u{7f}\u{85}/雪🦀.so";
        assert_eq!(toml_string(value), r#""C:\\tools\\\"backend\"\b\t\n\f\r\u0001\u007f\u0085/雪🦀.so""#);
    }

    #[test]
    fn cranelift_requires_an_explicit_absolute_backend_path() {
        for backend in [
            None,
            Some(OsString::new()),
            Some(OsString::from("cranelift")),
            Some(OsString::from("relative/backend.so")),
            Some(std::env::current_dir().unwrap().into_os_string()),
            Some(std::env::current_exe().unwrap().join("missing-backend.so").into_os_string()),
        ] {
            for args in [vec!["--engine=cranelift", "build"], vec!["--engine=cranelift", "plan", "build"]] {
                let result = plan(
                    &request(&args),
                    Programs { cargo: "cargo".into(), lust: "lust".into(), explicit_cargo: false, cranelift_backend: backend.clone() },
                );
                assert!(result.unwrap_err().contains("LUST_CRANELIFT_BACKEND"));
            }
        }
    }

    #[test]
    fn non_cranelift_routes_do_not_require_backend_configuration() {
        for args in [
            vec!["--engine=cranelift", "build", "--release"],
            vec!["--engine=cranelift", "test", "--profile=custom"],
            vec!["--engine=cranelift", "check"],
            vec!["--engine=cranelift", "b"],
            vec!["--engine=cranelift", "r"],
            vec!["--engine=cranelift", "t"],
        ] {
            let plan = plan(
                &request(&args),
                Programs { cargo: "cargo".into(), lust: "lust".into(), explicit_cargo: false, cranelift_backend: None },
            )
            .unwrap();
            assert_eq!(plan.selected_engine, Engine::Rustc);
            assert!(plan.environment.is_empty());
            assert!(plan.codegen_backend.is_none());
        }
    }

    #[test]
    fn explicit_profiles_and_release_do_not_change() {
        for profile in [
            vec!["--release"],
            vec!["-r"],
            vec!["-vr"],
            vec!["-rv"],
            vec!["-qr"],
            vec!["--profile", "custom"],
            vec!["--profile=custom"],
            vec!["--profile", "dev"],
        ] {
            let mut args = vec!["--engine", "cranelift", "build"];
            args.extend(profile);
            let plan = prepared(&args);
            assert_eq!(plan.selected_engine, Engine::Rustc);
            assert_eq!(plan.args, words(&args[2..]));
            assert!(plan.environment.is_empty());
            assert!(plan.codegen_backend.is_none());
        }
    }

    #[test]
    fn short_option_values_are_not_release_flags() {
        for argument in ["-pserver", "-vrpserver", "-Ffeature", "-Zflag"] {
            let plan = prepared(&["--engine=cranelift", "build", argument]);
            assert_eq!(plan.selected_engine, if argument == "-vrpserver" { Engine::Rustc } else { Engine::Cranelift });
        }
    }

    #[test]
    fn runtime_arguments_do_not_select_profile() {
        let plan = prepared(&["--engine", "cranelift", "run", "--", "--release", "--profile", "prod"]);
        assert_eq!(plan.selected_engine, Engine::Cranelift);
        let mut expected = words(&["run"]);
        expected.extend(backend_config_args());
        expected.extend(words(&["--", "--release", "--profile", "prod"]));
        assert_eq!(plan.args, expected);
    }

    #[test]
    fn check_bench_metadata_and_extensions_are_forwarded() {
        for command in ["check", "bench", "metadata", "doc", "clippy", "fmt", "clean", "rustc", "custom-subcommand", "b", "r", "t"] {
            let plan = prepared(&["--engine", "cranelift", command]);
            assert_eq!(plan.selected_engine, Engine::Rustc);
            assert!(plan.environment.is_empty());
            assert_eq!(plan.args, words(&[command]));
        }
    }

    #[test]
    fn lust_native_arguments_are_not_cargo_arguments() {
        let plan = prepared(&["--engine", "lust", "check", "source with spaces/main.rs", "--json"]);
        assert_eq!(plan.selected_engine, Engine::Lust);
        assert_eq!(plan.program, OsString::from("/tools/lust"));
        assert_eq!(plan.args, words(&["check", "source with spaces/main.rs", "--json"]));
        assert!(plan.codegen_backend.is_none());
        assert!(parse(&words(&["+nightly", "--engine", "lust", "build", "main.rs"])).is_err());
    }

    #[test]
    fn invalid_options_are_usage_errors() {
        for args in [
            vec!["--engine"],
            vec!["--engine", "magic", "build"],
            vec!["--engine", "rustc"],
            vec!["plan"],
            vec!["+stable", "+nightly", "build"],
            vec!["+", "build"],
            vec!["--"],
        ] {
            assert!(parse(&words(&args)).is_err(), "{args:?}");
        }
    }

    #[test]
    fn cargo_options_can_be_explicitly_forwarded() {
        let plan = prepared(&["--", "--version"]);
        assert_eq!(plan.args, words(&["--version"]));
    }

    #[test]
    fn json_escapes_path_and_arguments() {
        let plan = prepared(&["build", "--manifest-path", "a\"b\\c\n\r\t\u{1}/Cargo.toml"]);
        let json = plan.json();
        assert!(json.contains(r#""program":"/tools with spaces/cargo""#));
        assert!(json.contains(r#"a\"b\\c\n\r\t\u0001/Cargo.toml"#));
        assert!(!json.contains('\n'));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_arguments_are_preserved_in_execution_and_plan() {
        use std::os::unix::ffi::OsStringExt;
        let path = OsString::from_vec(vec![b'f', 0xff, b'o']);
        let args = vec![OsString::from("build"), OsString::from("--manifest-path"), path.clone()];
        let Action::Execute(req) = parse(&args).unwrap() else { panic!() };
        let plan = plan(
            &req,
            Programs { cargo: OsString::from("cargo"), lust: OsString::from("lust"), explicit_cargo: false, cranelift_backend: None },
        )
        .unwrap();
        assert_eq!(plan.args[2], path);
        assert!(plan.json().contains(r#"{"unix_bytes_hex":"66ff6f"}"#));
    }
}
