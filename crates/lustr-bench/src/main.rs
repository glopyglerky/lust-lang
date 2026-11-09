//! Linux-only qualification runner. No shell, unsafe code, inherited environment or fallback.
#![forbid(unsafe_code)]
mod order;
mod resources;
mod stats;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File, FileTimes, OpenOptions},
    io::{Read, Write},
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::PermissionsExt,
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime},
};
type R<T> = Result<T, Box<dyn std::error::Error>>;
fn fail<T>(s: impl Into<String>) -> R<T> {
    Err(s.into().into())
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Arg {
    Text(String),
    Bytes { hex: String },
}
impl Arg {
    fn os(&self) -> R<OsString> {
        match self {
            Self::Text(s) => Ok(s.into()),
            Self::Bytes { hex } => {
                if hex.len() % 2 != 0 {
                    return fail("odd hex");
                }
                let mut v = Vec::new();
                for i in (0..hex.len()).step_by(2) {
                    v.push(u8::from_str_radix(hex.get(i..i + 2).ok_or("invalid hex")?, 16)?);
                }
                if v.contains(&0) {
                    return fail("NUL argument");
                }
                Ok(OsString::from_vec(v))
            }
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Cmd {
    argv: Vec<Arg>,
    expected_exit: i32,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Tool {
    path: PathBuf,
    sha256: String,
    version_args: Vec<Arg>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Gate {
    name: String,
    kind: String,
    command: Cmd,
    compare: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Lane {
    name: String,
    env: BTreeMap<String, Arg>,
    tools: Vec<Tool>,
    command: Cmd,
    selection: Cmd,
    evidence: Cmd,
    evidence_contains: Vec<String>,
    gates: Vec<Gate>,
    normalize: Vec<[String; 2]>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    schema: u32,
    #[serde(default)]
    resource_mode: resources::Mode,
    source: PathBuf,
    repository: String,
    revision: String,
    package: String,
    root_crate: String,
    profile: String,
    target: String,
    features: Vec<String>,
    semantic_settings: BTreeMap<String, String>,
    external_inputs: Vec<PathBuf>,
    scenario: String,
    force_root: Option<PathBuf>,
    snapshot_mtime_authorized: bool,
    pairs: usize,
    warmup_pairs: usize,
    seed: u64,
    bootstrap_resamples: usize,
    timeout_seconds: u64,
    output_limit_bytes: u64,
    lanes: Vec<Lane>,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn hash(p: &Path) -> R<String> {
    let mut f = File::open(p)?;
    let mut h = Sha256::new();
    let mut b = [0u8; 65536];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(hex(&h.finalize()))
}
fn walk(root: &Path, at: &Path, rows: &mut BTreeMap<String, String>) -> R<()> {
    let mut entries = fs::read_dir(at)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if at == root && e.file_name() == ".git" {
            continue;
        }
        let t = e.file_type()?;
        if t.is_symlink() {
            return fail(format!("source symlink requires a materialized snapshot: {}", p.display()));
        }
        if t.is_dir() {
            walk(root, &p, rows)?;
        } else if t.is_file() {
            rows.insert(
                hex(p.strip_prefix(root)?.as_os_str().as_bytes()),
                format!("{:o}:{}", e.metadata()?.permissions().mode(), hash(&p)?),
            );
        } else {
            return fail("special source file");
        }
    }
    Ok(())
}
fn source(s: &Spec) -> R<Value> {
    let mut rows = BTreeMap::new();
    walk(&s.source, &s.source, &mut rows)?;
    let bytes = serde_json::to_vec(&rows)?;
    let mut external = BTreeMap::new();
    for p in &s.external_inputs {
        external.insert(hex(p.as_os_str().as_bytes()), input_digest(p)?);
    }
    Ok(
        json!({"tree_sha256":hex(&Sha256::digest(bytes)),"files_hex_paths":rows,"external_inputs":external,"cargo_lock_sha256":hash(&s.source.join("Cargo.lock"))?}),
    )
}
fn input_digest(p: &Path) -> R<String> {
    if p.is_dir() {
        let mut rows = BTreeMap::new();
        walk(p, p, &mut rows)?;
        Ok(hex(&Sha256::digest(serde_json::to_vec(&rows)?)))
    } else {
        hash(p)
    }
}
struct ChildScope(std::process::Child);
impl Drop for ChildScope {
    fn drop(&mut self) {
        let _ = kill_group(self.0.id());
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn allowed(k: &str) -> bool {
    matches!(
        k,
        "PATH"
            | "HOME"
            | "TMPDIR"
            | "LANG"
            | "LC_ALL"
            | "TZ"
            | "CARGO_HOME"
            | "RUSTUP_HOME"
            | "RUSTC"
            | "RUSTDOC"
            | "CARGO"
            | "LUST_CARGO"
            | "RUSTFLAGS"
            | "CARGO_ENCODED_RUSTFLAGS"
            | "RUSTDOCFLAGS"
            | "RUSTC_WRAPPER"
            | "RUSTC_WORKSPACE_WRAPPER"
            | "CARGO_BUILD_JOBS"
            | "CARGO_BUILD_TARGET"
            | "CARGO_INCREMENTAL"
            | "CARGO_NET_OFFLINE"
            | "CARGO_TERM_COLOR"
            | "LUST_NO_DAEMON"
            | "LUST_NO_CACHE"
            | "LUST_CACHE_DIR"
            | "LUST_DAEMON_SOCKET"
            | "LUST_CRANELIFT_BACKEND"
            | "CC"
            | "CXX"
            | "AR"
            | "CFLAGS"
            | "CXXFLAGS"
            | "PKG_CONFIG_PATH"
    ) || k.starts_with("CARGO_PROFILE_")
        || k.starts_with("CARGO_TARGET_")
}
fn env(l: &Lane, target: &Path) -> R<BTreeMap<OsString, OsString>> {
    let mut e = BTreeMap::new();
    for (k, v) in &l.env {
        if !allowed(k) || k == "CARGO_TARGET_DIR" {
            return fail(format!("environment key not allowlisted: {k}"));
        }
        e.insert(k.into(), v.os()?);
    }
    e.insert("CARGO_TARGET_DIR".into(), target.as_os_str().into());
    Ok(e)
}
fn host_context() -> R<Value> {
    let load = fs::read_to_string("/proc/loadavg")?;
    let load: Vec<_> = load.split_whitespace().take(3).map(str::to_owned).collect();
    let stat = fs::read_to_string("/proc/stat")?;
    let cpu: Vec<_> = stat.lines().filter(|line| line.starts_with("cpu")).map(str::to_owned).collect();
    let status = fs::read_to_string("/proc/self/status")?;
    let affinity =
        status.lines().find_map(|line| line.strip_prefix("Cpus_allowed_list:")).ok_or("CPU affinity observation missing")?.trim();
    if load.len() != 3 || cpu.is_empty() || affinity.is_empty() {
        return fail("incomplete host observation");
    }
    Ok(json!({
        "cpu_affinity":affinity,
        "load_average":load,
        "cpu_time_counters":cpu,
        "cpu_pressure":fs::read_to_string("/proc/pressure/cpu")?,
        "io_pressure":fs::read_to_string("/proc/pressure/io")?,
    }))
}

#[derive(Serialize)]
struct Record {
    lane: String,
    phase: String,
    pair: Option<usize>,
    argv: Vec<Arg>,
    wall_seconds: f64,
    exit: Option<i32>,
    signal: Option<i32>,
    timed_out: bool,
    output_limit: bool,
    stdout: String,
    stderr: String,
    stdout_sha256: String,
    stderr_sha256: String,
    stdout_bytes: u64,
    stderr_bytes: u64,
    host_before: Value,
    host_after: Value,
    resources: Value,
}
struct Runner {
    dir: PathBuf,
    records: Vec<Record>,
}
impl Runner {
    fn run(&mut self, s: &Spec, l: &Lane, target: &Path, c: &Cmd, phase: &str, pair: Option<usize>) -> R<(f64, Vec<u8>, Vec<u8>)> {
        let args = c.argv.iter().map(Arg::os).collect::<R<Vec<_>>>()?;
        if args.is_empty() {
            return fail("empty argv");
        }
        let exe = PathBuf::from(&args[0]);
        if !exe.is_absolute() || !l.tools.iter().any(|t| t.path == exe) {
            return fail("command executable must be absolute and listed in tools");
        }
        let id = self.records.len();
        let outname = format!("{id:05}.stdout");
        let errname = format!("{id:05}.stderr");
        let op = self.dir.join(&outname);
        let ep = self.dir.join(&errname);
        let out = OpenOptions::new().write(true).create_new(true).open(&op)?;
        let err = OpenOptions::new().write(true).create_new(true).open(&ep)?;
        let host_before = host_context()?;
        let usage_path = self.dir.join(format!("{id:05}.usage.json"));
        let request_path = self.dir.join(format!("{id:05}.request.json"));
        let enabled = s.resource_mode == resources::Mode::LinuxWaited;
        let mut binding = None;
        let mut command = if enabled {
            let collector = std::env::current_exe()?;
            let collector_hash = hash(&collector)?;
            let encode = |v: OsString| Arg::Bytes { hex: hex(v.as_bytes()) };
            let request = resources::Request {
                argv: c.argv.clone(),
                env: env(l, target)?.into_iter().map(|(k, v)| (encode(k), encode(v))).collect(),
                cwd: s.source.clone(),
                collector_sha256: collector_hash.clone(),
                report_path: usage_path.clone(),
            };
            let bytes = serde_json::to_vec(&request)?;
            publish(&request_path, &bytes)?;
            binding = Some((hex(&Sha256::digest(&bytes)), collector_hash));
            let mut command = Command::new(collector);
            command.arg("--internal-resource-collector-v2").arg(&request_path).arg(&usage_path);
            command
        } else {
            let mut command = Command::new(exe);
            command.args(&args[1..]);
            command
        };
        let start = Instant::now();
        let child = command
            .current_dir(&s.source)
            .env_clear()
            .envs(if enabled { BTreeMap::new() } else { env(l, target)? })
            .stdin(if enabled { Stdio::piped() } else { Stdio::null() })
            .stdout(out)
            .stderr(err)
            .process_group(0)
            .spawn()?;
        let mut scope = ChildScope(child);
        let child = &mut scope.0;
        let _lease = child.stdin.take();
        let pid = child.id();
        let mut timeout = false;
        let mut limit = false;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            timeout = start.elapsed() > Duration::from_secs(s.timeout_seconds);
            limit = fs::metadata(&op)?.len().saturating_add(fs::metadata(&ep)?.len()) > s.output_limit_bytes;
            if timeout || limit {
                kill_group(pid)?;
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        let seconds = start.elapsed().as_secs_f64();
        // Kill lingering descendants even when the direct child succeeded.
        kill_group(pid)?;
        let host_after = host_context()?;
        let ob = fs::metadata(&op)?.len();
        let eb = fs::metadata(&ep)?.len();
        limit |= ob.saturating_add(eb) > s.output_limit_bytes;
        let oh = hash(&op)?;
        let eh = hash(&ep)?;
        let mut resource_error = None;
        let mut payload_status = status;
        let resource = if let Some((request_hash, collector_hash)) = binding {
            let usage = if timeout || limit {
                fail("command interrupted: resources unavailable")
            } else {
                resources::read(&usage_path, &request_hash, &collector_hash, pid, status)
            };
            match usage {
                Ok(u) => {
                    payload_status = std::process::ExitStatus::from_raw(u.raw_status);
                    json!({"availability":"available","collector_exit":status.code(),"collector_signal":status.signal(),"report":format!("{id:05}.usage.json"),"report_sha256":hash(&usage_path)?,"usage":u})
                }
                Err(e) => {
                    resource_error = Some(e.to_string());
                    json!({"availability":"unavailable","collector_exit":status.code(),"collector_signal":status.signal(),"scope":resources::SCOPE,"error":e.to_string(),"usage":null,"raw_report":format!("{id:05}.usage.json")})
                }
            }
        } else {
            json!({"availability":"disabled","usage":null})
        };
        self.records.push(Record {
            lane: l.name.clone(),
            phase: phase.into(),
            pair,
            argv: c.argv.clone(),
            wall_seconds: seconds,
            exit: if resource_error.is_some() { None } else { payload_status.code() },
            signal: if resource_error.is_some() { None } else { payload_status.signal() },
            timed_out: timeout,
            output_limit: limit,
            stdout: outname,
            stderr: errname,
            stdout_sha256: oh,
            stderr_sha256: eh,
            stdout_bytes: ob,
            stderr_bytes: eb,
            host_before,
            host_after,
            resources: resource,
        });
        if resource_error.is_some() || timeout || limit || payload_status.code() != Some(c.expected_exit) {
            return fail(format!("{} {phase}: command failed; raw evidence retained", l.name));
        }
        Ok((seconds, fs::read(op)?, fs::read(ep)?))
    }
}
fn kill_group(pid: u32) -> R<()> {
    // Host compatibility utility; no shell or unsafe FFI. Negative operand names process group.
    let status = Command::new("/bin/kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    // ESRCH is normal after the entire group exits. Other failures are caught by child.wait timeout path.
    let _ = status;
    Ok(())
}
fn normalize(l: &Lane, raw: &[u8]) -> Vec<u8> {
    // Literal byte substitutions only. No scripts, regexes, or lossy UTF-8 decoding.
    let mut out = raw.to_vec();
    for [a, b] in &l.normalize {
        let a = a.as_bytes();
        if a.is_empty() {
            continue;
        }
        let mut next = Vec::new();
        let mut i = 0;
        while i < out.len() {
            if out[i..].starts_with(a) {
                next.extend_from_slice(b.as_bytes());
                i += a.len();
            } else {
                next.push(out[i]);
                i += 1;
            }
        }
        out = next;
    }
    out
}
fn metadata_for_comparison(l: &Lane, raw: &[u8]) -> R<(Value, Vec<Value>)> {
    let mut value: Value = serde_json::from_slice(&normalize(l, raw))?;
    if value.get("version") != Some(&json!(1)) {
        return fail("metadata comparison requires Cargo format version 1");
    }
    let mut changes = Vec::new();
    for package in value.get_mut("packages").and_then(Value::as_array_mut).ok_or("metadata packages must be an array")? {
        let id = package.get("id").cloned().unwrap_or(Value::Null);
        if let Some(hints) = package.get_mut("hints").and_then(Value::as_object_mut)
            && hints.get("min-opt-level").is_some_and(Value::is_null)
        {
            hints.remove("min-opt-level");
            changes.push(json!({"package":id,"field":"/hints/min-opt-level","observed":null,"comparison":"absent"}));
        }
    }
    Ok((value, changes))
}
fn tools(l: &Lane) -> R<Value> {
    let mut rows = Vec::new();
    for t in &l.tools {
        let h = hash(&t.path)?;
        if h != t.sha256 {
            return fail(format!("tool identity changed: {}", t.path.display()));
        }
        rows.push(json!({"path":t.path,"sha256":h}));
    }
    Ok(json!(rows))
}
fn touch(s: &Spec) -> R<()> {
    if let Some(p) = &s.force_root {
        let f = OpenOptions::new().write(true).open(s.source.join(p))?;
        f.set_times(FileTimes::new().set_modified(SystemTime::now()))?;
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}
fn canonical_source(p: &Path) -> R<bool> {
    let own = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    let home_repos = std::env::var_os("HOME").and_then(|home| fs::canonicalize(PathBuf::from(home).join("repo")).ok());
    Ok(p.starts_with(own) || home_repos.is_some_and(|repos| p.starts_with(repos)))
}
fn cargo_tail(c: &Cmd) -> R<Vec<OsString>> {
    let args = c.argv.iter().map(Arg::os).collect::<R<Vec<_>>>()?;
    let mut i = 1;
    if args.get(i).is_some_and(|a| a == "lustr" || a == "lust") {
        i += 1;
    }
    while let Some(arg) = args.get(i) {
        if ["rustc", "build", "check", "test", "run", "metadata"].iter().any(|op| arg == op) {
            return Ok(args[i..].to_vec());
        }
        if ["--bluefe-root", "--frontend-jobs", "--engine", "--color"].iter().any(|option| arg == option) {
            if args.get(i + 1).is_none() {
                return fail("missing selector/global option value");
            }
            i += 2;
        } else if arg.to_str().is_some_and(|a| {
            a.starts_with("--frontend-jobs=") || a.starts_with("--engine=") || a.starts_with("--color=") || a.starts_with('+')
        }) || ["-v", "-vv", "--verbose", "-q", "--quiet", "--locked", "--offline", "--frozen"].iter().any(|option| arg == option)
        {
            i += 1;
        } else {
            return fail("unrecognized command prefix; use an explicit Cargo/lustr operation, without planning or aliases");
        }
    }
    fail("Cargo operation absent in command")
}
fn rustc_check(args: &[OsString]) -> bool {
    args.first().is_some_and(|a| a == "rustc")
        && args
            .iter()
            .take_while(|a| *a != "--")
            .enumerate()
            .any(|(i, a)| a == "--profile=check" || (a == "--profile" && args.get(i + 1).is_some_and(|v| v == "check")))
}
fn future_canonical(path: &Path) -> R<PathBuf> {
    if !path.is_absolute() || path.components().any(|p| matches!(p, std::path::Component::ParentDir)) {
        return fail("resolved Cargo directory must be absolute without parent traversal");
    }
    let mut ancestor = path;
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => {
                let mut p = fs::canonicalize(ancestor)?;
                for name in suffix.iter().rev() {
                    p.push(name);
                }
                return Ok(p);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                suffix.push(ancestor.file_name().ok_or("missing directory ancestor")?.to_owned());
                ancestor = ancestor.parent().ok_or("missing directory parent")?;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
fn validate_build_directories(metadata: &Value, target: &Path) -> R<()> {
    let target = fs::canonicalize(target)?;
    for key in ["target_directory", "build_directory"] {
        let value = metadata
            .get(key)
            .and_then(Value::as_str)
            .ok_or("Cargo metadata must report target_directory and build_directory for isolation")?;
        if !future_canonical(Path::new(value))?.starts_with(&target) {
            return fail(format!("resolved Cargo {key} escapes its owned lane directory"));
        }
    }
    Ok(())
}
fn validate(s: &Spec, out: &Path) -> R<()> {
    if s.schema != 1
        || s.lanes.len() < 2
        || s.lanes.len() > 8
        || s.pairs < 8
        || s.pairs > 10000
        || s.pairs % 2 != 0
        || s.warmup_pairs < 1
        || s.warmup_pairs > 100
        || !(100..=1_000_000).contains(&s.bootstrap_resamples)
        || s.timeout_seconds == 0
        || s.timeout_seconds > 86400
        || s.output_limit_bytes < 1024
        || s.output_limit_bytes > 256 * 1024 * 1024
    {
        return fail("invalid schema/counts/limits (pairs must be even and >=8)");
    }
    if canonical_source(&s.source)? {
        return fail("qualification requires a frozen snapshot outside canonical repositories");
    }
    if s.source != fs::canonicalize(&s.source)? || !s.source.is_absolute() || out.starts_with(&s.source) || s.source.starts_with(out) {
        return fail("source and output must be canonical, disjoint absolute paths");
    }
    if s.root_crate.is_empty() || s.profile.is_empty() || s.target.is_empty() || s.semantic_settings.is_empty() {
        return fail("missing reproducibility settings");
    }
    match s.scenario.as_str() {
        "no-op" | "primed-root-check" | "primed-root-build" | "compile-plus-test" | "clean-dependency-build" => (),
        _ => return fail("unknown scenario"),
    }
    if s.scenario.starts_with("primed-root") && s.force_root.is_none() {
        return fail("primed root mode needs force_root");
    }
    if let Some(p) = &s.force_root {
        if !s.snapshot_mtime_authorized || p.is_absolute() || p.components().any(|x| !matches!(x, std::path::Component::Normal(_))) {
            return fail("force_root needs authorized snapshot and safe relative path");
        }
        if canonical_source(&s.source)? {
            return fail("canonical source metadata writes forbidden");
        }
        if !s.source.join(p).is_file() {
            return fail("force root is not a file");
        }
    }
    let mut names = BTreeSet::new();
    if s.scenario == "no-op" && s.force_root.is_some() {
        return fail("no-op cannot force a root rebuild");
    }
    for l in &s.lanes {
        if l.name.is_empty() || !l.name.bytes().all(|x| x.is_ascii_alphanumeric() || x == b'-') || !names.insert(&l.name) {
            return fail("invalid/duplicate lane name");
        }
        env(l, &out.join("targets").join(&l.name))?;
        tools(l)?;
        let words = cargo_tail(&l.command)?;
        let op = &words[0];
        let checking = op == "check" || rustc_check(&words);
        let building = op == "build" || (op == "rustc" && !checking);
        let valid = match s.scenario.as_str() {
            "primed-root-check" => checking,
            "primed-root-build" | "clean-dependency-build" => building,
            "compile-plus-test" => op == "test" && !words.iter().any(|a| a == "--no-run"),
            "no-op" => checking || building,
            _ => false,
        };
        if !valid {
            return fail("timed Cargo operation disagrees with scenario label");
        }
        let jobs = l.env.get("CARGO_BUILD_JOBS").and_then(|a| a.os().ok()).and_then(|v| v.to_str().and_then(|s| s.parse::<usize>().ok()));
        let available = std::thread::available_parallelism()?.get();
        if jobs.is_none_or(|jobs| jobs == 0 || jobs > available) {
            return fail("explicit positive Cargo build budget must fit the runner's available CPU affinity");
        }
        for k in ["RUSTC", "RUSTDOC", "CARGO"] {
            let p = l.env.get(k).ok_or(format!("missing {k}"))?.os()?;
            if !l.tools.iter().any(|t| t.path.as_os_str() == p) {
                return fail(format!("{k} must be hash-pinned in tools"));
            }
        }
        if !l.gates.iter().any(|g| g.kind == "runtime" || g.kind == "tests")
            || !l.gates.iter().any(|g| g.kind == "rejection" && g.command.expected_exit != 0)
        {
            return fail("each lane needs real runtime/tests and rejection gates");
        }
        for g in &l.gates {
            if !matches!(g.compare.as_str(), "stdout" | "stderr" | "both") {
                return fail("gate compare must be stdout/stderr/both");
            }
        }
        if l.normalize.iter().any(|[a, _]| a.is_empty()) {
            return fail("empty normalization");
        }
        if l.command.expected_exit != 0 || l.selection.expected_exit != 0 || l.evidence.expected_exit != 0 {
            return fail("measurement/metadata/evidence must succeed");
        }
        if l.evidence_contains.iter().any(|v| v.is_empty()) || l.evidence_contains.is_empty() {
            return fail("explicit actual compiler flag evidence required");
        }
    }
    for l in &s.lanes {
        let commands = std::iter::once(&l.command).chain([&l.selection, &l.evidence]).chain(l.gates.iter().map(|g| &g.command));
        for c in commands {
            for a in &c.argv {
                let os = a.os()?;
                let b = os.as_bytes();
                if b == b"--target-dir"
                    || b.starts_with(b"--target-dir=")
                    || b == b"--build-dir"
                    || b.starts_with(b"--build-dir=")
                    || b == b"--config"
                    || b.starts_with(b"--config=")
                    || b == b"-j"
                    || b.starts_with(b"--jobs")
                    || (b.starts_with(b"-j") && b.len() > 2)
                {
                    return fail("target/config/jobs argv overrides forbidden; set allowlisted env and use source Cargo config");
                }
            }
        }
        if l.evidence_contains != s.lanes[0].evidence_contains {
            return fail("shared semantic compiler flag evidence must agree across lanes");
        }
    }
    let first = &s.lanes[0].gates;
    for l in &s.lanes[1..] {
        let semantics = |lane: &Lane| -> R<BTreeMap<String, Vec<u8>>> {
            lane.env
                .iter()
                .filter(|(k, _)| {
                    k.starts_with("CARGO_PROFILE_")
                        || k.starts_with("CARGO_TARGET_")
                        || matches!(
                            k.as_str(),
                            "RUSTFLAGS"
                                | "CARGO_ENCODED_RUSTFLAGS"
                                | "RUSTDOCFLAGS"
                                | "CARGO_INCREMENTAL"
                                | "CARGO_BUILD_TARGET"
                                | "CC"
                                | "CXX"
                                | "AR"
                                | "CFLAGS"
                                | "CXXFLAGS"
                                | "RUSTC_WRAPPER"
                                | "RUSTC_WORKSPACE_WRAPPER"
                        )
                })
                .map(|(k, v)| Ok((k.clone(), v.os()?.as_bytes().to_vec())))
                .collect()
        };
        for (a, b) in [(&l.command, &s.lanes[0].command), (&l.evidence, &s.lanes[0].evidence), (&l.selection, &s.lanes[0].selection)] {
            if cargo_tail(a)? != cargo_tail(b)? {
                return fail("Cargo operation/profile/target/features/flags differ between lanes");
            }
        }
        if semantics(l)? != semantics(&s.lanes[0])? {
            return fail("lane semantic settings/wrappers differ");
        }
        if l.gates.len() != first.len()
            || l.gates.iter().zip(first).any(|(a, b)| {
                a.name != b.name || a.kind != b.kind || a.compare != b.compare || a.command.expected_exit != b.command.expected_exit
            })
        {
            return fail("gates differ between lanes");
        }
    }
    Ok(())
}
fn evidence(r: &mut Runner, s: &Spec, l: &Lane, target: &Path, phase: &str) -> R<()> {
    touch(s)?;
    let (_, _, err) = r.run(s, l, target, &l.evidence, phase, None)?;
    verify_evidence(s, l, &err)?;
    Ok(())
}
fn verify_evidence(s: &Spec, l: &Lane, err: &[u8]) -> R<()> {
    let lines = err.split(|b| *b == b'\n').filter(|line| line.windows(7).any(|w| w == b"Running")).collect::<Vec<_>>();
    let marker = format!("--crate-name {} ", s.root_crate);
    let compiler = l.env["RUSTC"].os()?;
    let args = l.command.argv.iter().map(Arg::os).collect::<R<Vec<_>>>()?;
    let prefix = args.len() - cargo_tail(&l.command)?.len();
    let mut frontend = None;
    for i in 1..prefix {
        if args[i] == "--frontend-jobs" {
            frontend = args.get(i + 1).and_then(|v| v.to_str()).map(str::to_owned);
        } else if let Some(v) = args[i].to_str().and_then(|v| v.strip_prefix("--frontend-jobs=")) {
            frontend = Some(v.to_owned());
        }
    }
    let has_arg = |line: &[u8], arg: &str| {
        line.windows(arg.len()).enumerate().any(|(i, w)| {
            w == arg.as_bytes()
                && (i == 0 || b" \t'\"`".contains(&line[i - 1]))
                && (i + arg.len() == line.len() || b" \t'\"`".contains(&line[i + arg.len()]))
        })
    };
    if !lines.iter().any(|line| {
        line.windows(marker.len()).any(|w| w == marker.as_bytes())
            && line.windows(compiler.as_bytes().len()).any(|w| w == compiler.as_bytes())
            && l.evidence_contains.iter().all(|x| line.windows(x.len()).any(|w| w == x.as_bytes()))
            && (s.scenario != "primed-root-check"
                || ((has_arg(line, "--emit=dep-info,metadata") || has_arg(line, "--emit=metadata"))
                    && line.windows(7).filter(|w| *w == b"--emit=").count() == 1))
            && frontend.as_ref().is_none_or(|jobs| has_arg(line, &format!("--jobs-frontend={jobs}")))
    }) {
        return fail("actual root compiler command/flags or metadata-only emission absent from Cargo verbose evidence");
    }
    Ok(())
}

fn study(r: &mut Runner, s: &Spec, report: &mut Value) -> R<()> {
    let before = source(s)?;
    report["source_before"] = before.clone();
    let mut dependency_roots = BTreeSet::new();
    let mut dependency_before = BTreeMap::new();
    let mut selections = Vec::new();
    let mut metadata_projections = Vec::new();
    let mut gates: Vec<Vec<Vec<u8>>> = Vec::new();
    let mut identities = Vec::new();
    for l in &s.lanes {
        let target = r.dir.join("targets").join(&l.name);
        fs::create_dir_all(&target)?;
        let mut versions = Vec::new();
        for t in &l.tools {
            let mut argv = vec![Arg::Text(t.path.to_str().ok_or("non-UTF8 tool path: use UTF8 tool snapshot")?.into())];
            argv.extend(t.version_args.clone());
            let (_, stdout, stderr) = r.run(s, l, &target, &Cmd { argv, expected_exit: 0 }, "identity", None)?;
            versions.push(json!({"tool":t.path,"sha256":t.sha256,"stdout_hex":hex(&stdout),"stderr_hex":hex(&stderr)}));
        }
        identities.push(json!({"lane":l.name,"tools":versions}));
        report["identities"] = json!(identities);
        let (_, selection, _) = r.run(s, l, &target, &l.selection, "dependency-selection", None)?;
        let parsed: Value = serde_json::from_slice(&selection)?;
        if parsed.get("packages").is_none() || parsed.get("resolve").is_none_or(Value::is_null) {
            return fail("selection must be complete cargo metadata JSON with resolved dependencies");
        }
        validate_build_directories(&parsed, &target)?;
        for package in parsed["packages"].as_array().ok_or("packages array missing")? {
            let manifest = PathBuf::from(package["manifest_path"].as_str().ok_or("manifest path missing")?);
            let root = fs::canonicalize(manifest.parent().ok_or("dependency root")?)?;
            if !root.starts_with(&s.source) && dependency_roots.insert(root.clone()) {
                dependency_before.insert(hex(root.as_os_str().as_bytes()), input_digest(&root)?);
            }
        }
        report["dependency_sources_before"] = json!(dependency_before);
        let (comparison, changes) = metadata_for_comparison(l, &selection)?;
        selections.push(comparison);
        metadata_projections.push(json!({"lane":l.name,"changes":changes}));
        report["metadata_comparison_projections"] = json!(metadata_projections);
        evidence(r, s, l, &target, "compiler-evidence-before")?;
        let mut values = Vec::new();
        for g in &l.gates {
            let (_, stdout, stderr) = r.run(s, l, &target, &g.command, &format!("gate:{}", g.name), None)?;
            let v = match g.compare.as_str() {
                "stdout" => normalize(l, &stdout),
                "stderr" => normalize(l, &stderr),
                _ => serde_json::to_vec(&(normalize(l, &stdout), normalize(l, &stderr)))?,
            };
            values.push(v);
        }
        gates.push(values);
    }
    report["compiler_or_cargo_identity_confounders"] = json!(
        s.lanes[1..]
            .iter()
            .map(|l| {
                let base = &s.lanes[0];
                ["RUSTC", "CARGO"]
                    .iter()
                    .filter_map(|key| {
                        let a = base.env[*key].os().ok()?;
                        let b = l.env[*key].os().ok()?;
                        let ah = base.tools.iter().find(|t| t.path.as_os_str() == a)?.sha256.clone();
                        let bh = l.tools.iter().find(|t| t.path.as_os_str() == b)?.sha256.clone();
                        if ah != bh { Some(format!("{}: {key} binary differs from baseline; versions retained", l.name)) } else { None }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    );
    if selections.iter().any(|x| *x != selections[0]) {
        return fail("normalized actual dependency selection differs");
    }
    if gates.iter().any(|x| *x != gates[0]) {
        return fail("normalized correctness results differ");
    }
    let mut times: Vec<Vec<f64>> = (0..s.lanes.len()).map(|_| Vec::with_capacity(s.pairs)).collect();
    let mut order = Vec::new();
    // Blocks restart at each phase; one seeded RNG stream binds both schedules.
    let mut rng = s.seed;
    for warm in [true, false] {
        let count = if warm { s.warmup_pairs } else { s.pairs };
        for (pair, indices) in order::rounds(s.lanes.len(), count, &mut rng).into_iter().enumerate() {
            order.push(json!({"warmup":warm,"pair":pair,"lane_indices":indices}));
            report["order"] = json!(order);
            for i in indices {
                let l = &s.lanes[i];
                let target = r.dir.join("targets").join(&l.name);
                if s.scenario == "clean-dependency-build" {
                    fs::remove_dir_all(&target)?;
                    fs::create_dir(&target)?;
                }
                touch(s)?;
                let (elapsed, _, stderr) = r.run(s, l, &target, &l.command, if warm { "excluded-warmup" } else { "timed" }, Some(pair))?;
                if s.force_root.is_some() {
                    verify_evidence(s, l, &stderr)?;
                }
                if !warm {
                    times[i].push(elapsed);
                }
                if source(s)? != before {
                    return fail("source/external inputs mutated during run");
                }
                tools(l)?;
            }
        }
    }
    // Recheck runtime/rejection after timed work and retain actual compiler evidence.
    for (i, l) in s.lanes.iter().enumerate() {
        let target = r.dir.join("targets").join(&l.name);
        if s.force_root.is_none() {
            fs::remove_dir_all(&target)?;
            fs::create_dir(&target)?;
        }
        evidence(r, s, l, &target, "compiler-evidence-after")?;
        for (j, g) in l.gates.iter().enumerate() {
            let (_, o, e) = r.run(s, l, &target, &g.command, &format!("final-gate:{}", g.name), None)?;
            let v = match g.compare.as_str() {
                "stdout" => normalize(l, &o),
                "stderr" => normalize(l, &e),
                _ => serde_json::to_vec(&(normalize(l, &o), normalize(l, &e)))?,
            };
            if v != gates[i][j] {
                return fail("final correctness result changed");
            }
        }
        tools(l)?;
    }
    let mut dependency_after = BTreeMap::new();
    for root in dependency_roots {
        dependency_after.insert(hex(root.as_os_str().as_bytes()), input_digest(&root)?);
    }
    report["dependency_sources_after"] = json!(dependency_after);
    if dependency_before != dependency_after {
        return fail("selected dependency source changed");
    }
    let after = source(s)?;
    report["source_after"] = after.clone();
    if before != after {
        return fail("source final guard failed");
    }
    let mut fingerprints = BTreeMap::new();
    for l in &s.lanes {
        let root = r.dir.join("targets").join(&l.name);
        let mut rows = BTreeMap::new();
        fingerprint_files(&root, &root, &mut rows)?;
        if rows.is_empty() {
            return fail("no observed Cargo fingerprints inside owned lane directory");
        }
        fingerprints.insert(l.name.clone(), rows);
    }
    report["actual_cargo_fingerprints"] = json!(fingerprints);
    if times.iter().any(|lane| lane.len() != s.pairs) {
        return fail("incomplete paired sample cardinality");
    }
    let mut summaries = Vec::new();
    for i in 1..s.lanes.len() {
        let pairs: Vec<_> = times[0].iter().copied().zip(times[i].iter().copied()).collect();
        summaries.push(json!({"baseline":s.lanes[0].name,"candidate":s.lanes[i].name,"analysis":stats::summarize(&pairs,s.bootstrap_resamples,s.seed)?}));
    }
    report["summaries"] = json!(summaries);
    Ok(())
}
fn fingerprint_files(root: &Path, at: &Path, rows: &mut BTreeMap<String, Value>) -> R<()> {
    for e in fs::read_dir(at)? {
        let e = e?;
        let p = e.path();
        let t = e.file_type()?;
        if t.is_dir() {
            fingerprint_files(root, &p, rows)?;
        } else if t.is_file()
            && p.extension().is_some_and(|s| s == "json")
            && p.components().any(|c| c.as_os_str() == ".fingerprint" || c.as_os_str() == "fingerprint")
        {
            let bytes = fs::read(&p)?;
            rows.insert(
                hex(p.strip_prefix(root)?.as_os_str().as_bytes()),
                json!({"sha256":hex(&Sha256::digest(&bytes)),"data":serde_json::from_slice::<Value>(&bytes)?}),
            );
        }
    }
    Ok(())
}

fn publish(path: &Path, bytes: &[u8]) -> R<()> {
    let tmp = path.with_extension("pending");
    let mut f = OpenOptions::new().create_new(true).write(true).open(&tmp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    fs::hard_link(&tmp, path)?;
    fs::remove_file(tmp)?;
    File::open(path.parent().ok_or("no parent")?)?.sync_all()?;
    Ok(())
}
fn main() -> R<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "--internal-resource-collector-v2") {
        if args.len() != 4 {
            return fail("invalid collector invocation");
        }
        std::process::exit(resources::collect(Path::new(&args[2]), Path::new(&args[3]))?);
    }
    if args.len() != 3 {
        return fail("usage: lustr-bench SPEC.json NEW-ABSOLUTE-REPORT-DIRECTORY");
    }
    let bytes = fs::read(&args[1])?;
    let s: Spec = serde_json::from_slice(&bytes)?;
    let requested = PathBuf::from(&args[2]);
    if !requested.is_absolute() {
        return fail("output must be absolute");
    }
    let parent = fs::canonicalize(requested.parent().ok_or("output parent")?)?;
    let out = parent.join(requested.file_name().ok_or("output name")?);
    validate(&s, &out)?;
    fs::create_dir(&out)?;
    publish(&out.join("spec.original.json"), &bytes)?;
    let mut r = Runner { dir: out.clone(), records: Vec::new() };
    let mut report = json!({"schema":1,"runner_version":env!("CARGO_PKG_VERSION"),"runner_sha256":hash(&std::env::current_exe()?)?,"spec_sha256":hex(&Sha256::digest(&bytes)),"spec":s,"status":"failed","summaries":null,"order_method":order::METHOD,"metadata_comparison_rule":"cargo-optional-min-opt-level-none-v1","metadata_comparison_projections":[],"memory_energy":"not measured","inference_assumptions":"equivalent work; independent pairs for bootstrap; sign exchangeability under null; exploratory unadjusted comparisons"});
    report["resource_measurement"] = resources::metadata(s.resource_mode);
    if s.resource_mode != resources::Mode::Disabled {
        report["memory_energy"] = json!(
            "per-command CPU/largest waited-process peak RSS collection enabled; availability in records; aggregate RSS and energy not measured"
        );
    }
    let metadata = s.force_root.as_ref().map(|p| fs::metadata(s.source.join(p))).transpose()?;
    let result = study(&mut r, &s, &mut report);
    let restored = if let (Some(p), Some(m)) = (&s.force_root, metadata) {
        File::options()
            .write(true)
            .open(s.source.join(p))
            .and_then(|f| f.set_times(FileTimes::new().set_modified(m.modified()?).set_accessed(m.accessed()?)))
    } else {
        Ok(())
    };
    let mut error = result.err().map(|e| e.to_string()).or_else(|| restored.err().map(|e| format!("metadata restoration failed: {e}")));
    report["records"] = json!(r.records);
    report["final_source_observation"] = match source(&s) {
        Ok(v) => {
            if report.get("source_before").is_some_and(|b| *b != v) {
                error.get_or_insert("final source guard changed".into());
            }
            v
        }
        Err(e) => {
            error.get_or_insert(e.to_string());
            json!({"error":e.to_string()})
        }
    };
    let mut final_tools = Vec::new();
    for l in &s.lanes {
        match tools(l) {
            Ok(v) => final_tools.push(v),
            Err(e) => {
                error.get_or_insert(e.to_string());
                final_tools.push(json!({"error":e.to_string()}));
            }
        }
    }
    report["final_tools_observation"] = json!(final_tools);
    if let Some(e) = &error {
        report["error"] = json!(e);
        report["summaries"] = Value::Null;
    } else {
        report["status"] = json!("passed");
    }
    publish(&out.join("report.json"), &serde_json::to_vec_pretty(&report)?)?;
    println!("{}", out.join("report.json").display());
    if error.is_some() {
        return fail("qualification failed; report retained without conclusions");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_known_unset_metadata_hint_is_projected() {
        let s: Spec = serde_json::from_str(include_str!("../sample-spec.json")).unwrap();
        let l = &s.lanes[0];
        let old = json!({"version":1,"packages":[{"id":"example","hints":{"mostly-unused":true}}],"resolve":{"features":["default"]}});
        let mut unset = old.clone();
        unset["packages"][0]["hints"]["min-opt-level"] = Value::Null;
        let project = |v: &Value| metadata_for_comparison(l, &serde_json::to_vec(v).unwrap()).unwrap();
        assert_eq!(project(&old).0, project(&unset).0);
        assert_eq!(project(&unset).1.len(), 1);
        assert_eq!(project(&old).1.len(), 0);
        for v in [json!(0), json!(1), json!(2), json!(3), json!("s"), json!(false), json!({})] {
            let mut material = unset.clone();
            material["packages"][0]["hints"]["min-opt-level"] = v;
            assert_ne!(project(&old).0, project(&material).0);
        }
        for material in [
            json!({"version":1,"packages":[{"id":"example","hints":{"mostly-unused":false}}],"resolve":{"features":["default"]}}),
            json!({"version":1,"packages":[{"id":"example","hints":{"mostly-unused":true,"future-hint":null}}],"resolve":{"features":["default"]}}),
            json!({"version":1,"packages":[{"id":"example","hints":{"mostly-unused":true}}],"resolve":{"features":["extra"]}}),
        ] {
            assert_ne!(project(&old).0, project(&material).0);
        }
        let mut future = old.clone();
        future["version"] = json!(2);
        assert!(metadata_for_comparison(l, &serde_json::to_vec(&future).unwrap()).is_err());
    }
    #[test]
    fn os_bytes_and_normalization() {
        assert_eq!(Arg::Bytes { hex: "61ff62".into() }.os().unwrap().as_bytes(), &[b'a', 255, b'b']);
        for h in ["a", "gg", "00", "é"] {
            assert!(Arg::Bytes { hex: h.into() }.os().is_err());
        }
        let s: Spec = serde_json::from_str(include_str!("../sample-spec.json")).unwrap();
        let mut l = s.lanes[0].clone();
        l.normalize = vec![["/lane".into(), "<T>".into()]];
        assert_eq!(normalize(&l, b"\xff/lane/a/lane"), b"\xff<T>/a<T>");
        l.env.insert("SECRET_TOKEN".into(), Arg::Text("not logged".into()));
        assert!(env(&l, Path::new("/output")).is_err());
    }
    #[test]
    fn actual_evidence_required() {
        let s: Spec = serde_json::from_str(include_str!("../sample-spec.json")).unwrap();
        let l = &s.lanes[0];
        assert!(verify_evidence(&s, l, b"requested --crate-name bench_tiny").is_err());
        assert!(verify_evidence(&s, l, b"Running `/toolchain/bin/rustc --crate-name other --edition=2024 -C debuginfo=2`").is_err());
        assert!(verify_evidence(&s, l, b"Running `/toolchain/bin/rustc --crate-name bench_tiny --edition=2024 -C debuginfo=2`").is_err());
        assert!(
            verify_evidence(
                &s,
                l,
                b"Running `/toolchain/bin/rustc --crate-name bench_tiny --emit=dep-info,metadata,link --edition=2024 -C debuginfo=2`"
            )
            .is_err()
        );
        assert!(
            verify_evidence(
                &s,
                l,
                b"Running `/toolchain/bin/rustc --crate-name bench_tiny --emit=dep-info,metadata --edition=2024 -C debuginfo=2`"
            )
            .is_ok()
        );
    }
    #[test]
    fn rustc_profile_values_do_not_hide_features_or_package_options() {
        let command = |args: &[&str]| Cmd { argv: args.iter().map(|s| Arg::Text((*s).into())).collect(), expected_exit: 0 };
        let base = command(&["/toolchain/cargo", "rustc", "-p", "core", "--profile", "check", "--lib"]);
        let changed = command(&[
            "/toolchain/lustr",
            "--bluefe-root",
            "/toolchain",
            "--frontend-jobs=4",
            "rustc",
            "--features",
            "extra",
            "-p",
            "core",
            "--profile",
            "check",
            "--lib",
        ]);
        let tail = cargo_tail(&base).unwrap();
        assert_eq!(tail[0], "rustc");
        assert!(rustc_check(&tail));
        assert_ne!(tail, cargo_tail(&changed).unwrap());
        let same = command(&[
            "/toolchain/lustr",
            "--bluefe-root",
            "check",
            "--frontend-jobs",
            "4",
            "rustc",
            "-p",
            "core",
            "--profile",
            "check",
            "--lib",
        ]);
        assert_eq!(tail, cargo_tail(&same).unwrap());
        assert!(cargo_tail(&command(&["/toolchain/lustr", "plan", "check"])).is_err());
    }
    #[test]
    fn cargo_build_outputs_must_stay_in_the_owned_lane() {
        let nonce = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("benchmark-isolation-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let lane = root.join("lane");
        fs::create_dir(&lane).unwrap();
        let shared = root.join("shared");
        fs::create_dir(&shared).unwrap();
        let good = json!({"target_directory":lane,"build_directory":lane.join("new/build")});
        assert!(validate_build_directories(&good, &lane).is_ok());
        let bad = json!({"target_directory":lane,"build_directory":shared});
        assert!(validate_build_directories(&bad, &lane).is_err());
        std::os::unix::fs::symlink(&shared, lane.join("redirect")).unwrap();
        let escaped = json!({"target_directory":lane,"build_directory":lane.join("redirect")});
        assert!(validate_build_directories(&escaped, &lane).is_err());
        assert!(validate_build_directories(&json!({"target_directory":lane}), &lane).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn old_and_new_cargo_fingerprint_layouts_are_observed() {
        let nonce = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("benchmark-fingerprints-{}-{nonce}", std::process::id()));
        for p in ["debug/.fingerprint/pkg/lib-pkg.json", "debug/build/pkg/hash/fingerprint/lib-pkg.json"] {
            let path = root.join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"{\"profile\":1}").unwrap();
        }
        let mut rows = BTreeMap::new();
        fingerprint_files(&root, &root, &mut rows).unwrap();
        assert_eq!(rows.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
