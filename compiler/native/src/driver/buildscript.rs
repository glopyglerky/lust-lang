// Build-script refresh checks (docs/ARCH.md section 1.1). Lust does not
// compile `build.rs`; Cargo compiles and runs it (the probe's `cargo
// check`), and the driver reads the recorded output. A record is valid
// only while everything its run read still holds (`Record::holds`): the
// script crate's sources (its compile's dep-info), every
// `rerun-if-changed` path (a dir with all its files) or, when the script
// prints no rerun-if directive, every file of the package (Cargo's
// default), the manifests, lock file, Cargo config and toolchain files,
// every `rerun-if-env-changed` value and every var the script's compile
// read, the setup (target, profile, rustc, the client's Cargo vars), and
// OUT_DIR's contents. `refresh` re-checks them per session (hashes only,
// no Cargo run while they hold) and re-runs the script through Cargo when
// one changed. Env values and the Cargo run use the requesting client's
// env (`ClientEnv`), never the daemon's, as Cargo started from the
// client's shell would. The outputs hash covers `cargo::rustc-cfg`,
// `cargo::rustc-env`, and the contents of OUT_DIR files; it enters the
// daemon's per-session refresh key and the cold disk cache key, so a
// build script change can never serve output resolved under the old one.
// `outputs` gives the recorded values themselves to the driver, which
// feeds them to expansion (`env!`, OUT_DIR includes) and to the crate cfg
// env (`rustc-cfg`).
use crate::intern::FxHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// The env a build script check runs under: the requesting client's
/// whole env (`protocol::Client::env`), or None for this process's own
/// env (one-shot compiles, where this process is the client, and tests).
pub type ClientEnv<'a> = Option<&'a [(String, String)]>;

/// One var of `env` as the client sees it.
fn client_var(env: ClientEnv, name: &str) -> Option<String> {
    match env {
        Some(env) => env.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()),
        None => std::env::var(name).ok(),
    }
}

/// The client's vars that steer Cargo's builds, sorted: `CARGO_*` (not the
/// terminal's or the jobserver's) and rustc's (`RUSTC*`, `RUSTFLAGS`,
/// `RUSTDOCFLAGS`, `RUSTUP_TOOLCHAIN`).
fn cargo_vars(env: ClientEnv) -> Vec<(String, String)> {
    let keep = |k: &str| {
        (k.starts_with("CARGO_") && !k.starts_with("CARGO_TERM_") && k != "CARGO_MAKEFLAGS")
            || k.starts_with("RUSTC")
            || matches!(k, "RUSTFLAGS" | "RUSTDOCFLAGS" | "RUSTUP_TOOLCHAIN")
    };
    let mut vars: Vec<(String, String)> = match env {
        Some(env) => env.iter().filter(|(k, _)| keep(k)).cloned().collect(),
        None => std::env::vars().filter(|(k, _)| keep(k)).collect(),
    };
    vars.sort();
    vars
}

/// Build script stdout directives the driver records. Cargo accepts
/// both `cargo:key=value` and `cargo::key=value`; other lines are the
/// script's own diagnostics and are ignored here, as Cargo ignores
/// them for build purposes.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct BuildOutput {
    pub rerun_if_changed: Vec<String>,
    pub rerun_if_env_changed: Vec<String>,
    pub rustc_cfg: Vec<String>,
    pub rustc_check_cfg: Vec<String>,
    pub rustc_env: Vec<(String, String)>,
}

/// Parses build script stdout into `BuildOutput`.
pub fn parse_output(text: &str) -> BuildOutput {
    let mut out = BuildOutput::default();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("cargo:") else { continue };
        let rest = rest.strip_prefix(':').unwrap_or(rest);
        let Some((key, value)) = rest.split_once('=') else { continue };
        match key {
            "rerun-if-changed" => out.rerun_if_changed.push(value.to_string()),
            "rerun-if-env-changed" => out.rerun_if_env_changed.push(value.to_string()),
            "rustc-cfg" => out.rustc_cfg.push(value.to_string()),
            "rustc-check-cfg" => out.rustc_check_cfg.push(value.to_string()),
            "rustc-env" => {
                let (k, v) = value.split_once('=').unwrap_or((value, ""));
                out.rustc_env.push((k.to_string(), v.to_string()));
            }
            _ => {}
        }
    }
    out
}

/// One recorded build-script run: what its outputs depend on, re-checked
/// per session, and the outputs consumers read. Stored as text lines in
/// the disk cache dir (magic `LBS4`; `LBS3` lacked the script's compile,
/// and `LBS2` the script's other sources, the manifests, a rerun-if-changed
/// dir's files, Cargo's default package files, the setup and the OUT_DIR
/// re-check); W-DRIVER
/// owns the form and changes it only by bumping the magic, never by
/// migrating an old record (an old record reads as a miss and the script
/// re-runs).
pub struct Record {
    pub script: PathBuf,
    /// The script crate's sources (the script, its modules, `include!`d
    /// files: its compile's dep-info) and their hashes.
    pub sources: Vec<(PathBuf, u64)>,
    /// The files and dirs the run read (`Input`).
    pub inputs: Vec<Input>,
    /// Vars and their values in the client's env (None: unset): each
    /// `rerun-if-env-changed`, and each var the script's compile read that
    /// Cargo does not set itself.
    pub env_inputs: Vec<(String, Option<String>)>,
    /// rustc as Cargo gave it to the script (`RUSTC`), when Cargo printed it.
    pub rustc: Option<PathBuf>,
    /// `setup_hash` of the run.
    pub setup: u64,
    pub output_hash: u64,
    /// The OUT_DIR Cargo gave this run, when Cargo reported one.
    pub out_dir: Option<PathBuf>,
    /// The recorded directives (`rustc-cfg`, `rustc-env`, ...).
    pub output: BuildOutput,
    /// The script's compile, when Cargo printed it and a replay made a binary.
    pub compile: Option<Compile>,
}

/// A build script's compile as Cargo ran it (the dir, the vars Cargo set,
/// rustc and its args), and the hash of the binary a `replay` of it made.
/// When only the script crate's sources changed, a replay that makes the
/// same binary means the run would do the same, so the record stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compile {
    pub cwd: PathBuf,
    pub vars: Vec<(String, String)>,
    pub argv: Vec<String>,
    pub bin: u64,
}

/// What a stored record says of the client's state now (`Record::holds`).
#[derive(Debug, PartialEq, Eq)]
enum Holds {
    /// Everything the run read still holds.
    All,
    /// Only the script crate's sources changed, and a replay can tell.
    OnlySources,
    /// Something else changed: the script runs again.
    No,
}

/// A file-system input of a record and its hash at the run: a path (a
/// file, or a dir with all its files, as Cargo reads a `rerun-if-changed`
/// dir; `path_hash`), or the package dir (Cargo's default input for a
/// script with no rerun-if directive; `package_hash`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub package: bool,
    pub path: PathBuf,
    pub hash: u64,
}

fn hash_parts(parts: &[&[u8]]) -> u64 {
    let mut h = FxHasher::default();
    "buildscript-v1".hash(&mut h);
    for p in parts {
        (p.len() as u64).hash(&mut h);
        p.hash(&mut h);
    }
    h.finish()
}

/// Hashes recorded outputs the way consumers key on them: cfgs, env
/// pairs, and OUT_DIR file contents (names and bytes, sorted).
fn output_hash(out: &BuildOutput, out_dir: Option<&Path>) -> u64 {
    let mut parts: Vec<Vec<u8>> = vec![b"cfg".to_vec()];
    let mut cfgs = out.rustc_cfg.clone();
    cfgs.sort();
    parts.extend(cfgs.iter().map(|c| c.as_bytes().to_vec()));
    parts.push(b"checkcfg".to_vec());
    let mut checks = out.rustc_check_cfg.clone();
    checks.sort();
    parts.extend(checks.iter().map(|c| c.as_bytes().to_vec()));
    parts.push(b"env".to_vec());
    let mut envs = out.rustc_env.clone();
    envs.sort();
    parts.extend(envs.iter().map(|(k, v)| format!("{k}={v}").into_bytes()));
    if let Some(dir) = out_dir {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else {
                        files.push(p);
                    }
                }
            }
        }
        files.sort();
        for f in files {
            parts.push(f.to_string_lossy().as_bytes().to_vec());
            parts.push(std::fs::read(&f).unwrap_or_default());
        }
    }
    let refs: Vec<&[u8]> = parts.iter().map(|v| &v[..]).collect();
    hash_parts(&refs)
}

/// The record file for one manifest dir inside the cache dir.
fn record_path(cache_dir: &Path, manifest_dir: &Path) -> PathBuf {
    let h = hash_parts(&[manifest_dir.to_string_lossy().as_bytes()]);
    cache_dir.join(format!("buildscript-{h:016x}.txt"))
}

/// Every build script record in `cache_dir` (any package). The daemon's
/// guard counts them as compile inputs: a rerun rewrites its record.
pub fn record_files(cache_dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(cache_dir) else { return Vec::new() };
    rd.flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("buildscript-") && n.ends_with(".txt")))
        .collect()
}

/// File contents hash for rerun-if inputs; a missing file hashes as a
/// fixed sentinel, so creating or deleting the file changes the hash.
fn file_hash(path: &Path) -> u64 {
    match std::fs::read(path) {
        Ok(bytes) => hash_parts(&[&bytes]),
        Err(_) => hash_parts(&[b"<missing>"]),
    }
}

/// The files under `dir` (names relative to it, and their `file_hash`,
/// sorted) hashed, without the entries `skip` takes (a skipped dir is not
/// entered). Symlinked dirs are not followed.
fn dir_hash(dir: &Path, skip: &dyn Fn(&Path) -> bool) -> u64 {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if skip(&p) {
                continue;
            }
            match e.file_type() {
                Ok(t) if t.is_dir() => stack.push(p),
                Ok(_) => files.push(p),
                Err(_) => {}
            }
        }
    }
    files.sort();
    let mut parts: Vec<Vec<u8>> = vec![b"dir".to_vec()];
    for f in &files {
        parts.push(f.strip_prefix(dir).unwrap_or(f).to_string_lossy().as_bytes().to_vec());
        parts.push(file_hash(f).to_le_bytes().to_vec());
    }
    let refs: Vec<&[u8]> = parts.iter().map(|v| &v[..]).collect();
    hash_parts(&refs)
}

/// A `rerun-if-changed` path hashed as Cargo reads it: a file's bytes, or a
/// dir's files (all of them); a missing path hashes as `file_hash`'s sentinel.
fn path_hash(path: &Path) -> u64 {
    if path.is_dir() { dir_hash(path, &|_| false) } else { file_hash(path) }
}

/// Every file of the package at `dir` hashed (`dir_hash`), for a script that
/// prints no rerun-if directive: Cargo's default reruns it on any package
/// file change. Left out as Cargo leaves them out of a package: VCS dirs,
/// nested packages and target dirs (`CACHEDIR.TAG`); and lust's own files
/// (`.lust*`, the cache dir). More files than Cargo's list only rerun more.
fn package_hash(dir: &Path, cache_dir: &Path) -> u64 {
    dir_hash(dir, &|p: &Path| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        matches!(name, ".git" | ".hg" | ".svn")
            || name.starts_with(".lust")
            || p == cache_dir
            || (p.is_dir() && (p.join("CACHEDIR.TAG").exists() || p.join("Cargo.toml").exists()))
    })
}

/// A file's size and modification time, the way a toolchain update shows.
fn stamp(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(m) => format!("{} {:?}", m.len(), m.modified().ok()),
        Err(_) => "<missing>".to_string(),
    }
}

/// The setup a run had, hashed: the target and profile the probe builds
/// for (the host's dev profile: `cargo check` without `--target`), rustc
/// (its path and stamp), and the client's vars that steer Cargo
/// (`cargo_vars`).
fn setup_hash(rustc: Option<&Path>, env: ClientEnv) -> u64 {
    let mut parts: Vec<Vec<u8>> = vec![b"target=host profile=dev".to_vec()];
    if let Some(r) = rustc {
        parts.push(r.to_string_lossy().as_bytes().to_vec());
        parts.push(stamp(r).into_bytes());
    }
    parts.extend(cargo_vars(env).into_iter().map(|(k, v)| format!("{k}={v}").into_bytes()));
    let refs: Vec<&[u8]> = parts.iter().map(|v| &v[..]).collect();
    hash_parts(&refs)
}

/// The files besides the script crate's own that steer its compile and
/// run, to hash as inputs (missing ones too, so creating one counts): the
/// package's Cargo.toml; the workspace's Cargo.lock and Cargo.toml (in the
/// nearest dir up that has a Cargo.lock, else the package's); in every dir
/// from the package up, Cargo's config and rustup's toolchain files; and
/// Cargo's config in CARGO_HOME.
fn setup_files(manifest_dir: &Path, env: ClientEnv) -> Vec<PathBuf> {
    let mut files = vec![manifest_dir.join("Cargo.toml")];
    let root = manifest_dir.ancestors().find(|d| d.join("Cargo.lock").exists()).unwrap_or(manifest_dir);
    files.push(root.join("Cargo.lock"));
    if root != manifest_dir {
        files.push(root.join("Cargo.toml"));
    }
    for d in manifest_dir.ancestors() {
        files.extend([d.join(".cargo/config.toml"), d.join(".cargo/config"), d.join("rust-toolchain.toml"), d.join("rust-toolchain")]);
    }
    let home = client_var(env, "CARGO_HOME").map(PathBuf::from).or_else(|| client_var(env, "HOME").map(|h| Path::new(&h).join(".cargo")));
    if let Some(home) = home {
        files.extend([home.join("config.toml"), home.join("config")]);
    }
    files
}

/// Escapes a free text field of a record line (a path or a value) so it
/// stays on its line: backslash, newline and carriage return.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r")
}

/// Reads a field written by `esc`.
fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

impl Record {
    /// Serializes the record. Paths are lossy text; a record that
    /// fails to read back is a miss, never a wrong answer.
    fn encode(&self) -> Vec<u8> {
        let p = |p: &Path| esc(&p.to_string_lossy());
        let mut s = String::from("LBS4\n");
        s.push_str(&format!("script {}\n", p(&self.script)));
        for (path, h) in &self.sources {
            s.push_str(&format!("src {h} {}\n", p(path)));
        }
        for i in &self.inputs {
            s.push_str(&format!("{} {} {}\n", if i.package { "package" } else { "input" }, i.hash, p(&i.path)));
        }
        for (k, v) in &self.env_inputs {
            // No value token at all means the var was unset;
            // an empty value is a real, different state.
            match v {
                Some(v) => s.push_str(&format!("env {k} {}\n", esc(v))),
                None => s.push_str(&format!("env {k}\n")),
            }
        }
        if let Some(r) = &self.rustc {
            s.push_str(&format!("rustc {}\n", p(r)));
        }
        s.push_str(&format!("setup {}\n", self.setup));
        if let Some(dir) = &self.out_dir {
            s.push_str(&format!("outdir {}\n", p(dir)));
        }
        for c in &self.output.rustc_cfg {
            s.push_str(&format!("cfg {}\n", esc(c)));
        }
        for c in &self.output.rustc_check_cfg {
            s.push_str(&format!("checkcfg {}\n", esc(c)));
        }
        for (k, v) in &self.output.rustc_env {
            s.push_str(&format!("renv {k} {}\n", esc(v)));
        }
        s.push_str(&format!("output {}\n", self.output_hash));
        if let Some(c) = &self.compile {
            s.push_str(&format!("cwd {}\n", p(&c.cwd)));
            for (k, v) in &c.vars {
                s.push_str(&format!("cvar {k} {}\n", esc(v)));
            }
            for a in &c.argv {
                s.push_str(&format!("arg {}\n", esc(a)));
            }
            s.push_str(&format!("bin {}\n", c.bin));
        }
        s.into_bytes()
    }

    /// Reads bytes written by `encode`. None on any malformed line.
    fn decode(text: &str) -> Option<Record> {
        let mut lines = text.lines();
        if lines.next()? != "LBS4" {
            return None;
        }
        let (mut script, mut setup, mut output_hash) = (None, None, None);
        let (mut sources, mut inputs, mut env_inputs) = (Vec::new(), Vec::new(), Vec::new());
        let (mut rustc, mut out_dir, mut output) = (None, None, BuildOutput::default());
        let (mut cwd, mut vars, mut argv, mut bin) = (None, Vec::new(), Vec::new(), None);
        for line in lines {
            let (tag, rest) = line.split_once(' ')?;
            // `<hash> <text>` and `<key> [<text>]` lines.
            let pair = || rest.split_once(' ').map(|(a, b)| (a, unesc(b)));
            match tag {
                "script" => script = Some(PathBuf::from(unesc(rest))),
                "src" => {
                    let (h, path) = pair()?;
                    sources.push((PathBuf::from(path), h.parse().ok()?));
                }
                "input" | "package" => {
                    let (h, path) = pair()?;
                    inputs.push(Input { package: tag == "package", path: PathBuf::from(path), hash: h.parse().ok()? });
                }
                "env" => env_inputs.push(match pair() {
                    Some((k, v)) => (k.to_string(), Some(v)),
                    None => (rest.to_string(), None),
                }),
                "rustc" => rustc = Some(PathBuf::from(unesc(rest))),
                "setup" => setup = Some(rest.parse().ok()?),
                "outdir" => out_dir = Some(PathBuf::from(unesc(rest))),
                "cfg" => output.rustc_cfg.push(unesc(rest)),
                "checkcfg" => output.rustc_check_cfg.push(unesc(rest)),
                "renv" => {
                    let (k, v) = pair().unwrap_or((rest, String::new()));
                    output.rustc_env.push((k.to_string(), v));
                }
                "output" => output_hash = Some(rest.parse().ok()?),
                "cwd" => cwd = Some(PathBuf::from(unesc(rest))),
                "cvar" => {
                    let (k, v) = pair().unwrap_or((rest, String::new()));
                    vars.push((k.to_string(), v));
                }
                "arg" => argv.push(unesc(rest)),
                "bin" => bin = Some(rest.parse().ok()?),
                _ => return None,
            }
        }
        let compile = cwd.zip(bin).filter(|_| !argv.is_empty()).map(|(cwd, bin)| Compile { cwd, vars, argv, bin });
        Some(Record {
            script: script?,
            sources,
            inputs,
            env_inputs,
            rustc,
            setup: setup?,
            output_hash: output_hash?,
            out_dir,
            output,
            compile,
        })
    }

    /// Whether everything the run read still holds in the client's `env`:
    /// the setup, each env input, each input path (the package's files
    /// hashed as `package_hash`, with `cache_dir` left out), OUT_DIR's
    /// contents with the recorded directives (a cleaned target dir is
    /// stale), and the script crate's sources.
    fn holds(&self, cache_dir: &Path, env: ClientEnv) -> Holds {
        let rest = self.setup == setup_hash(self.rustc.as_deref(), env)
            && self.env_inputs.iter().all(|(k, v)| client_var(env, k) == *v)
            && self.inputs.iter().all(|i| i.hash == if i.package { package_hash(&i.path, cache_dir) } else { path_hash(&i.path) })
            && output_hash(&self.output, self.out_dir.as_deref()) == self.output_hash;
        match (rest, self.sources.iter().all(|(p, h)| file_hash(p) == *h)) {
            (false, _) => Holds::No,
            (true, true) => Holds::All,
            (true, false) if self.compile.is_some() => Holds::OnlySources,
            (true, false) => Holds::No,
        }
    }
}

/// The build script for one manifest dir: `build = "..."` in the
/// `[package]` table when present (`false` means none), else the
/// conventional `build.rs`. A plain line scan, like protocol's manifest
/// reads; W-RESOLVE replaces it with Cargo metadata.
pub fn build_script_for(manifest_dir: &Path) -> Option<PathBuf> {
    if let Ok(text) = std::fs::read_to_string(manifest_dir.join("Cargo.toml")) {
        let mut in_package = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with('[') {
                in_package = line.trim_matches(|c| c == '[' || c == ']') == "package";
                continue;
            }
            if in_package && line.starts_with("build") {
                let value = line.split('=').nth(1)?.trim();
                if value == "false" {
                    return None;
                }
                let path = value.trim_matches(|c| c == '"' || c == '\'');
                let p = manifest_dir.join(path);
                return p.exists().then_some(p);
            }
        }
    }
    let p = manifest_dir.join("build.rs");
    p.exists().then_some(p)
}

/// What a refresh check found. `hash` is what session routing and cache
/// keys mix in; it is 0 when the crate has no build script.
pub enum Refresh {
    /// No build script, or no manifest dir to look in.
    None,
    /// Recorded inputs still hold; no Cargo run happened.
    Fresh { hash: u64 },
    /// The script re-ran through Cargo and the record was rewritten.
    Reran { hash: u64 },
    /// Inputs changed but the Cargo re-run failed. The caller logs it;
    /// the hash is over the failure text so each distinct failure state
    /// still separates sessions from the last good record.
    Failed { hash: u64, error: String },
}

impl Refresh {
    pub fn hash(&self) -> u64 {
        match self {
            Refresh::None => 0,
            Refresh::Fresh { hash } | Refresh::Reran { hash } | Refresh::Failed { hash, .. } => *hash,
        }
    }
}

/// Re-checks the build script of `manifest_dir` against its record in
/// `cache_dir`, re-running it through Cargo when stale. The fresh path
/// only hashes the recorded inputs (`Record::holds`), so a per-request
/// check stays cheap. One record per manifest dir serves every client: a
/// client whose `rerun-if-env-changed` values or Cargo vars differ from
/// the record's re-runs the script, as Cargo would from its shell.
pub fn refresh(manifest_dir: &Path, cache_dir: &Path, env: ClientEnv) -> Refresh {
    let Some(script) = build_script_for(manifest_dir) else { return Refresh::None };
    if let Some(rec) = fresh_record(manifest_dir, cache_dir, &script, env) {
        return Refresh::Fresh { hash: rec.output_hash };
    }
    match run_through_cargo(manifest_dir, &script, cache_dir, env) {
        Ok(rec) => {
            let hash = rec.output_hash;
            store_record(cache_dir, manifest_dir, &rec);
            Refresh::Reran { hash }
        }
        Err(e) => Refresh::Failed { hash: hash_parts(&[b"failed", e.as_bytes()]), error: e },
    }
}

/// The stored record for `manifest_dir` while everything it read still
/// holds in `env`. When only the script crate's sources changed (a comment,
/// say), the script's compile is replayed: the same binary means the same
/// run, so the record takes the new sources (and any var the compile now
/// reads) and stands.
fn fresh_record(manifest_dir: &Path, cache_dir: &Path, script: &Path, env: ClientEnv) -> Option<Record> {
    let path = record_path(cache_dir, manifest_dir);
    let mut rec = std::fs::read_to_string(&path).ok().and_then(|t| Record::decode(&t))?;
    if rec.script != script {
        return None;
    }
    match rec.holds(cache_dir, env) {
        Holds::All => Some(rec),
        Holds::No => None,
        Holds::OnlySources => {
            let c = rec.compile.as_ref()?;
            let (bin, sources, vars) = replay(c, rec.rustc.as_deref(), script, cache_dir, env)?;
            if bin != c.bin {
                return None;
            }
            rec.sources = source_hashes(sources, script);
            for k in vars.into_iter().filter(|k| !k.starts_with("CARGO")) {
                if !rec.env_inputs.iter().any(|(e, _)| *e == k) {
                    let v = client_var(env, &k);
                    rec.env_inputs.push((k, v));
                }
            }
            store_record(cache_dir, manifest_dir, &rec);
            Some(rec)
        }
    }
}

/// `sources` and the script itself (the one source known without the
/// compile's dep-info), with their hashes, each path once.
fn source_hashes(mut sources: Vec<PathBuf>, script: &Path) -> Vec<(PathBuf, u64)> {
    sources.push(script.to_path_buf());
    sources.sort();
    sources.dedup();
    sources
        .into_iter()
        .map(|p| {
            let h = file_hash(&p);
            (p, h)
        })
        .collect()
}

/// Compiles the script crate again as `c` says, without incremental state
/// and into a scratch dir under `cache_dir`, with the client's env plus the
/// vars Cargo set, and with rustc as Cargo gave it to the script when the
/// compile named a bare `rustc`: the binary's hash, and the sources and
/// vars the compile read (its dep-info). None when it fails.
fn replay(c: &Compile, rustc: Option<&Path>, script: &Path, cache_dir: &Path, env: ClientEnv) -> Option<(u64, Vec<PathBuf>, Vec<String>)> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
    let scratch = cache_dir.join(format!("buildscript-replay-{}-{nanos:x}", std::process::id()));
    std::fs::create_dir_all(&scratch).ok()?;
    let mut argv: Vec<String> = c.argv.first().into_iter().cloned().collect();
    let mut it = c.argv.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out-dir" => {
                it.next();
                argv.extend(["--out-dir".to_string(), scratch.to_string_lossy().into_owned()]);
            }
            "-C" => match it.next() {
                Some(v) if v.starts_with("incremental=") => {}
                v => argv.extend(std::iter::once(a.clone()).chain(v.cloned())),
            },
            _ if a.starts_with("-Cincremental=") => {}
            _ => argv.push(a.clone()),
        }
    }
    let program = match (argv.first(), rustc) {
        (Some(p), Some(r)) if !p.contains('/') => r.to_path_buf(),
        (p, _) => PathBuf::from(p?),
    };
    let mut cmd = std::process::Command::new(program);
    cmd.args(&argv[1..])
        .current_dir(&c.cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if let Some(env) = env {
        cmd.env_clear().envs(env.iter().map(|(k, v)| (k, v)));
    }
    cmd.envs(c.vars.iter().map(|(k, v)| (k, v)));
    let made = cmd.status().is_ok_and(|s| s.success()).then(|| {
        let bin = std::fs::read(scratch.join(format!("build_script_build{}", extra_filename(&argv)))).ok()?;
        let (_, sources, vars) = compile_dep_info(&argv, script)?;
        Some((hash_parts(&[&bin]), sources, vars))
    });
    let _ = std::fs::remove_dir_all(&scratch);
    made.flatten()
}

/// Writes one record to the cache dir (tmp file plus rename).
fn store_record(cache_dir: &Path, manifest_dir: &Path, rec: &Record) {
    let path = record_path(cache_dir, manifest_dir);
    let _ = std::fs::create_dir_all(cache_dir);
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    if std::fs::write(&tmp, rec.encode()).is_ok() && std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// The build script outputs expansion and cfg evaluation consume for
/// one package: its OUT_DIR and recorded directives.
#[derive(Default, Clone)]
pub struct Outputs {
    pub out_dir: Option<PathBuf>,
    pub rustc_cfg: Vec<String>,
    pub rustc_check_cfg: Vec<String>,
    pub rustc_env: Vec<(String, String)>,
}

/// The current outputs of the build script of `manifest_dir`: the
/// recorded ones while their inputs hold, else a fresh run through
/// Cargo (which also rewrites the record). Ok(None) means the package
/// has no build script. An Err means a script exists but its run
/// failed; the caller must not compile the crate without its outputs.
/// `env` is the requesting client's env, as for `refresh`.
pub fn outputs(manifest_dir: &Path, cache_dir: &Path, env: ClientEnv) -> Result<Option<Outputs>, String> {
    let Some(script) = build_script_for(manifest_dir) else { return Ok(None) };
    let rec = match fresh_record(manifest_dir, cache_dir, &script, env) {
        Some(rec) => rec,
        None => {
            let rec = run_through_cargo(manifest_dir, &script, cache_dir, env)?;
            store_record(cache_dir, manifest_dir, &rec);
            rec
        }
    };
    Ok(Some(Outputs {
        out_dir: rec.out_dir,
        rustc_cfg: rec.output.rustc_cfg,
        rustc_check_cfg: rec.output.rustc_check_cfg,
        rustc_env: rec.output.rustc_env,
    }))
}

/// Splits a command line as a POSIX shell does (Cargo's `-v` lines quote
/// words with `'...'`, `'\''` for a quote): words, quotes, backslashes.
fn shell_words(s: &str) -> Vec<String> {
    let (mut words, mut cur, mut in_word) = (Vec::new(), String::new(), false);
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                for c in chars.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    cur.push(c);
                }
            }
            '"' => {
                in_word = true;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => cur.extend(chars.next()),
                        c => cur.push(c),
                    }
                }
            }
            '\\' => {
                in_word = true;
                cur.extend(chars.next());
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            c => {
                in_word = true;
                cur.push(c);
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    words
}

/// The commands `cargo -vv` printed as it ran them (``Running `...` ``):
/// the vars Cargo set, and argv (a `lust-core` rustc wrapper left out).
fn printed_commands(stderr: &str) -> Vec<(Vec<(String, String)>, Vec<String>)> {
    let is_var = |w: &str| {
        w.split_once('=').is_some_and(|(k, _)| {
            !k.is_empty() && !k.starts_with(|c: char| c.is_ascii_digit()) && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
    };
    stderr
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("Running `")?.strip_suffix('`'))
        .map(|cmd| {
            let words = shell_words(cmd);
            let n = words.iter().take_while(|w| is_var(w)).count();
            let vars = words[..n].iter().filter_map(|w| w.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
            let mut argv: Vec<String> = words[n..].to_vec();
            if argv.first().is_some_and(|p| Path::new(p).file_name().is_some_and(|f| f == "lust-core")) {
                argv.remove(0);
            }
            (vars, argv)
        })
        .collect()
}

/// The `-C extra-filename=` value of a rustc command line ("" without one).
fn extra_filename(argv: &[String]) -> &str {
    argv.windows(2)
        .find_map(|w| (w[0] == "-C").then(|| w[1].strip_prefix("extra-filename=")).flatten())
        .or_else(|| argv.iter().find_map(|a| a.strip_prefix("-Cextra-filename=")))
        .unwrap_or("")
}

/// The dir Cargo ran a build script's compile in, and the source files and
/// env vars the compile read, from the dep-info rustc wrote beside it
/// (`--emit=dep-info`): `argv` is the compile as Cargo printed it, and
/// relative paths are joined to that dir (the one whose join with the
/// script argument is the script). None when the dep-info is not there or
/// that dir is not found.
fn compile_dep_info(argv: &[String], script: &Path) -> Option<(PathBuf, Vec<PathBuf>, Vec<String>)> {
    let arg = |name: &str| argv.windows(2).find(|w| w[0] == name).map(|w| w[1].as_str());
    let text = std::fs::read_to_string(Path::new(arg("--out-dir")?).join(format!("build_script_build{}.d", extra_filename(argv)))).ok()?;
    let src = argv.iter().skip(1).find(|a| !a.starts_with('-') && a.ends_with(".rs"))?;
    let script = std::fs::canonicalize(script).ok()?;
    let cwd = script.ancestors().skip(1).find(|d| d.join(src) == script)?.to_path_buf();
    // `<target>: <deps>` (spaces in a path escaped as `\ `), then a rule per
    // dep, then `# env-dep:NAME[=VALUE]` lines.
    let deps = text.lines().find_map(|l| l.split_once(": "))?.1;
    let (mut files, mut cur, mut chars) = (Vec::new(), String::new(), deps.chars());
    while let Some(c) = chars.next() {
        match c {
            '\\' => cur.extend(chars.next()),
            ' ' => files.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    files.push(cur);
    let files = files.into_iter().filter(|f| !f.is_empty()).map(|f| cwd.join(f)).collect();
    let vars =
        text.lines().filter_map(|l| l.strip_prefix("# env-dep:")).map(|v| v.split_once('=').map_or(v, |(k, _)| k).to_string()).collect();
    Some((cwd, files, vars))
}

/// The first lines of a failed `cargo -vv` run that are not Cargo's own
/// progress or echo (`Running`, `Fresh`, a script's output `[pkg ...]`).
fn cargo_error(stderr: &str) -> String {
    const ECHO: [&str; 9] = ["Running `", "Fresh ", "Compiling ", "Checking ", "Dirty ", "Finished ", "[", "Locking ", "Blocking "];
    let lines: Vec<&str> =
        stderr.lines().map(str::trim).filter(|l| !l.is_empty() && !ECHO.iter().any(|p| l.starts_with(p))).take(3).collect();
    lines.join(" ")
}

/// Runs the build script through Cargo and builds a fresh record.
/// `cargo check` already runs build scripts; its JSON messages name
/// the OUT_DIR, and the script's stdout sits in the sibling `output`
/// file Cargo keeps next to it. With `-vv`, Cargo also prints the
/// script's compile and run with the vars it set: the compile names its
/// dep-info (the script crate's sources, the vars it read) and the run
/// names rustc. The run is the cheap probe of `resolve::probe` (rustc runs
/// only for build scripts and proc macros). Cargo runs with the client's
/// whole `env` (Cargo passes its env on to the script), plus the vars the
/// probe command sets for itself.
fn run_through_cargo(manifest_dir: &Path, script: &Path, cache_dir: &Path, env: ClientEnv) -> Result<Record, String> {
    // Cargo decides script freshness by mtime; an input edited within
    // the same filesystem timestamp tick as the last run can look
    // fresh to it while our content hashes already moved. Rewriting
    // the script with its own bytes bumps its mtime and makes Cargo
    // execute it, which is the only outcome this path wants. The
    // record keys on contents, so the rewrite changes no hash.
    if let Ok(bytes) = std::fs::read(script) {
        let _ = std::fs::write(script, bytes);
    }
    let mut cmd = crate::resolve::probe::check_command(&manifest_dir.join("Cargo.toml"), "", env);
    if let Some(env) = env {
        let own: Vec<(std::ffi::OsString, std::ffi::OsString)> =
            cmd.get_envs().filter_map(|(k, v)| Some((k.to_owned(), v?.to_owned()))).collect();
        cmd.env_clear().envs(env.iter().map(|(k, v)| (k, v))).envs(own);
    }
    let out = cmd.arg("--offline").arg("-vv").env("CARGO_TERM_COLOR", "never").output().map_err(|e| format!("cannot run cargo: {e}"))?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        return Err(format!("cargo check failed: {}", cargo_error(&stderr)));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // One message per build script Cargo ran (this package and its
    // dependencies). The one for `manifest_dir` is the message whose
    // package_id names that directory (a path package id is
    // `path+file://<dir>#...`, `<dir>` percent-encoded); a single
    // message needs no match.
    let mut seen: Vec<(String, PathBuf)> = Vec::new();
    for line in stdout.lines() {
        if line.contains("\"build-script-executed\"") {
            // The message is one JSON object per line; pull the string
            // fields this record needs without a JSON parser.
            let field = |name: &str| -> Option<String> {
                // Skip past the whole `"name":"` opener (quotes, name, colon).
                let at = line.find(&format!("\"{name}\":\""))? + name.len() + 4;
                let rest = &line[at..];
                let mut s = String::new();
                let mut chars = rest.chars();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => return Some(s),
                        '\\' => s.push(chars.next()?),
                        _ => s.push(c),
                    }
                }
                None
            };
            if let Some(d) = field("out_dir") {
                seen.push((field("package_id").unwrap_or_default(), PathBuf::from(d)));
            }
        }
    }
    // The id's path is a percent-encoded URL (`sp%20ace`): decode it
    // before comparing with the manifest dir.
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let want = canon(manifest_dir);
    let out_dir = seen
        .iter()
        .find(|(id, _)| crate::resolve::cargo::path_package_dir(id).is_some_and(|d| d == manifest_dir || canon(&d) == want))
        .map(|(_, d)| d.clone())
        .or_else(|| if seen.len() == 1 { seen.into_iter().next().map(|(_, d)| d) } else { None });
    let Some(out_dir_path) = out_dir.filter(|d| !d.as_os_str().is_empty()) else {
        return Err("cargo check ran no build script for this manifest".to_string());
    };
    let output_file = out_dir_path.parent().map(|p| p.join("output"));
    let text = output_file.as_deref().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
    let parsed = parse_output(&text);
    // This package's script compile and run, as Cargo printed them.
    let printed = printed_commands(&stderr);
    let ours = |vars: &[(String, String)]| {
        vars.iter().any(|(k, v)| k == "CARGO_MANIFEST_DIR" && (Path::new(v) == manifest_dir || canon(Path::new(v)) == want))
    };
    let compile =
        printed.iter().find(|(vars, argv)| ours(vars) && argv.windows(2).any(|w| w[0] == "--crate-name" && w[1] == "build_script_build"));
    let run = printed.iter().find(|(vars, argv)| {
        ours(vars)
            && argv.first().is_some_and(|p| Path::new(p).file_name().is_some_and(|f| f.to_string_lossy().starts_with("build-script-")))
    });
    let rustc = run.and_then(|(vars, _)| vars.iter().find(|(k, _)| k == "RUSTC")).map(|(_, v)| PathBuf::from(v));
    let dep = compile.and_then(|(_, argv)| compile_dep_info(argv, script));
    let (sources, compile_vars) = dep.clone().map(|(_, f, v)| (f, v)).unwrap_or_default();
    // The compile, replayed once now: its binary's hash, for the next check.
    let compile = compile.zip(dep).and_then(|((vars, argv), (cwd, _, _))| {
        let mut c = Compile { cwd, vars: vars.clone(), argv: argv.clone(), bin: 0 };
        c.bin = replay(&c, rustc.as_deref(), script, cache_dir, env)?.0;
        Some(c)
    });
    // A rerun-if-changed path that is also a source of the script crate
    // (scripts list `build.rs` to turn off Cargo's package default) is checked
    // as a source: by its hash, or with a compile, by the binary a replay
    // makes. This takes it that a script does not read its own sources when
    // it runs; Cargo reruns the script when they change because it compiles
    // it again.
    let canon_sources: Vec<PathBuf> = if compile.is_some() { sources.iter().map(|p| canon(p)).collect() } else { Vec::new() };
    let mut inputs: Vec<Input> = parsed
        .rerun_if_changed
        .iter()
        .map(|p| if Path::new(p).is_absolute() { PathBuf::from(p) } else { manifest_dir.join(p) })
        .filter(|path| !canon_sources.contains(&canon(path)))
        .map(|path| Input { package: false, hash: path_hash(&path), path })
        .collect();
    // Cargo's default for a script with no rerun-if directive.
    if parsed.rerun_if_changed.is_empty() && parsed.rerun_if_env_changed.is_empty() {
        inputs.push(Input { package: true, path: manifest_dir.to_path_buf(), hash: package_hash(manifest_dir, cache_dir) });
    }
    inputs.extend(setup_files(manifest_dir, env).into_iter().map(|path| Input { package: false, hash: path_hash(&path), path }));
    // The vars the compile read that Cargo does not set (`CARGO*`).
    let mut env_inputs: Vec<(String, Option<String>)> = parsed
        .rerun_if_env_changed
        .iter()
        .chain(compile_vars.iter().filter(|k| !k.starts_with("CARGO")))
        .map(|k| (k.clone(), client_var(env, k)))
        .collect();
    env_inputs.sort();
    env_inputs.dedup();
    Ok(Record {
        output_hash: output_hash(&parsed, Some(&out_dir_path)),
        script: script.to_path_buf(),
        sources: source_hashes(sources, script),
        inputs,
        env_inputs,
        setup: setup_hash(rustc.as_deref(), env),
        rustc,
        out_dir: Some(out_dir_path),
        output: parsed,
        compile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both directive spellings parse, and unrecorded keys are ignored.
    #[test]
    fn output_directives_parse() {
        let out = parse_output(
            "cargo::rerun-if-changed=build.rs\ncargo:rerun-if-changed=data.txt\n\
             cargo::rerun-if-env-changed=GOOS\ncargo::rustc-cfg=wasm_pack\n\
             cargo::rustc-check-cfg=cfg(wasm_pack)\n\
             cargo:rustc-env=GREETING=hi there\ncargo:warning=ignored\nnoise\n",
        );
        assert_eq!(out.rerun_if_changed, ["build.rs", "data.txt"]);
        assert_eq!(out.rerun_if_env_changed, ["GOOS"]);
        assert_eq!(out.rustc_cfg, ["wasm_pack"]);
        assert_eq!(out.rustc_check_cfg, ["cfg(wasm_pack)"]);
        assert_eq!(out.rustc_env, [("GREETING".to_string(), "hi there".to_string())]);
    }

    /// A record round-trips, holds while its inputs are unchanged, and goes
    /// stale the moment one changes (the refresh decision that matters; no
    /// Cargo run in this test).
    #[test]
    fn record_holds_until_an_input_changes() {
        let dir = std::env::temp_dir().join(format!("lust-buildscript-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (script, data, assets, out) = (dir.join("build.rs"), dir.join("data.txt"), dir.join("assets"), dir.join("out"));
        std::fs::create_dir_all(&assets).unwrap();
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(&script, "fn main() {}").unwrap();
        std::fs::write(&data, "v1").unwrap();
        std::fs::write(assets.join("a.txt"), "one").unwrap();
        std::fs::write(out.join("gen.rs"), "pub const X: u8 = 1;").unwrap();
        // The client's env: the checks below change one var at a time.
        let base: Vec<(String, String)> = vec![("PATH".into(), "/bin".into())];
        let with = |k: &str, v: &str| base.iter().cloned().chain([(k.to_string(), v.to_string())]).collect::<Vec<_>>();
        let output = BuildOutput {
            rustc_cfg: vec!["wasm_pack".into(), "flavor=\"fast\"".into()],
            rustc_check_cfg: vec!["cfg(wasm_pack)".into()],
            rustc_env: vec![("GREETING".into(), "hi\nthere".into())],
            ..Default::default()
        };
        let rec = Record {
            script: script.clone(),
            sources: vec![(script.clone(), file_hash(&script))],
            inputs: vec![
                Input { package: false, hash: path_hash(&data), path: data.clone() },
                Input { package: false, hash: path_hash(&assets), path: assets.clone() },
            ],
            env_inputs: vec![("LUST_BS_TEST_ENV".into(), None)],
            rustc: None,
            setup: setup_hash(None, Some(&base)),
            output_hash: output_hash(&output, Some(&out)),
            out_dir: Some(out.clone()),
            output,
            compile: None,
        };
        let back = Record::decode(&String::from_utf8(rec.encode()).unwrap()).unwrap();
        assert_eq!(
            (back.sources.clone(), back.inputs.clone(), back.setup, back.output_hash),
            (rec.sources.clone(), rec.inputs.clone(), rec.setup, rec.output_hash)
        );
        assert_eq!((back.out_dir.clone(), back.output.clone()), (rec.out_dir.clone(), rec.output.clone()));
        let cache = dir.join("cache");
        assert_eq!(back.holds(&cache, Some(&base)), Holds::All, "unchanged inputs hold");
        // Env values come from the client's env, not this process's.
        assert_eq!(back.holds(&cache, Some(&with("LUST_BS_TEST_ENV", "1"))), Holds::No, "a client with the var set is stale");
        assert_eq!(back.holds(&cache, Some(&with("LUST_BS_TEST_ENV", ""))), Holds::No, "set to empty is not unset");
        assert_eq!(back.holds(&cache, Some(&with("RUSTFLAGS", "-Copt-level=1"))), Holds::No, "a client Cargo var is part of the setup");
        std::fs::write(&data, "v2").unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::No, "edited rerun-if-changed file is stale");
        std::fs::write(&data, "v1").unwrap();
        std::fs::write(assets.join("b.txt"), "two").unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::No, "a new file in a rerun-if-changed dir is stale");
        std::fs::remove_file(assets.join("b.txt")).unwrap();
        std::fs::write(assets.join("a.txt"), "uno").unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::No, "an edited file in a rerun-if-changed dir is stale");
        std::fs::write(assets.join("a.txt"), "one").unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::All, "the same contents again hold");
        std::fs::write(&script, "fn main() { }").unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::No, "an edited source is stale");
        std::fs::write(&script, "fn main() {}").unwrap();
        std::fs::remove_file(out.join("gen.rs")).unwrap();
        assert_eq!(back.holds(&cache, Some(&base)), Holds::No, "a file gone from OUT_DIR is stale");
        assert!(Record::decode("junk").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Real Cargo runs (a package with no dependencies, offline): the script
    /// re-runs exactly when something it read changed, and only then. The
    /// script appends a line to a log at each run.
    #[test]
    fn script_reruns_when_what_it_read_changes() {
        let root = std::env::temp_dir().join(format!("lust-buildscript-cargo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (pkg, cache, log) = (root.join("pkg"), root.join("cache"), root.join("runs.log"));
        std::fs::create_dir_all(pkg.join("src")).unwrap();
        std::fs::create_dir_all(pkg.join("assets")).unwrap();
        let manifest = "[package]\nname = \"bsrerun\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";
        std::fs::write(pkg.join("Cargo.toml"), manifest).unwrap();
        std::fs::write(pkg.join("src/main.rs"), "fn main() {}\n").unwrap();
        // `helper.rs` is a module of the script (its compile's dep-info names it), not a rerun-if-changed path.
        let script = "// v1: a plain comment.\nmod helper;\n/// Prints the directives.\nfn main() {\n    println!(\"cargo:rerun-if-changed=build.rs\");\n    println!(\"cargo:rerun-if-changed=data.txt\");\n    \
             println!(\"cargo:rerun-if-changed=assets\");\n    println!(\"cargo:rustc-cfg=bs_{}\", helper::tag());\n    \
             let data = std::fs::read_to_string(\"data.txt\").unwrap_or_default();\n    println!(\"cargo:rustc-env=BS_DATA={}\", data.trim());\n    \
             std::fs::write(std::path::Path::new(&std::env::var(\"OUT_DIR\").unwrap()).join(\"gen.txt\"), &data).unwrap();\n    \
             use std::io::Write;\n    let log = std::env::var(\"LUST_BS_TEST_LOG\").unwrap();\n    \
             std::fs::OpenOptions::new().create(true).append(true).open(log).unwrap().write_all(b\"run\\n\").unwrap();\n}\n";
        std::fs::write(pkg.join("build.rs"), script).unwrap();
        std::fs::write(pkg.join("helper.rs"), "// helper v1\npub fn tag() -> &'static str {\n    \"a\"\n}\n").unwrap();
        std::fs::write(pkg.join("data.txt"), "v1\n").unwrap();
        std::fs::write(pkg.join("assets/a.txt"), "one\n").unwrap();
        let env: Vec<(String, String)> =
            std::env::vars().chain([("LUST_BS_TEST_LOG".to_string(), log.to_string_lossy().into_owned())]).collect();
        let runs = |log: &Path| std::fs::read_to_string(log).map_or(0, |t| t.lines().count());
        let check = |pkg: &Path, log: &Path, want_runs: usize, what: &str| {
            let r = refresh(pkg, &cache, Some(&env));
            if let Refresh::Failed { error, .. } = &r {
                panic!("{what}: the run failed: {error}");
            }
            assert_eq!(runs(log), want_runs, "{what}");
        };
        let o = outputs(&pkg, &cache, Some(&env)).unwrap().unwrap();
        assert_eq!((o.rustc_cfg, o.rustc_env), (vec!["bs_a".to_string()], vec![("BS_DATA".to_string(), "v1".to_string())]));
        assert_eq!(runs(&log), 1, "the first build runs the script");
        check(&pkg, &log, 1, "nothing changed: no run");
        // Comments the compile drops: the same binary, so no run.
        let edited = script
            .replace("v1: a plain comment.", "v2: another comment, longer.")
            .replace("Prints the directives.", "Prints what Cargo reads.");
        std::fs::write(pkg.join("build.rs"), &edited).unwrap();
        check(&pkg, &log, 1, "a comment and a doc comment of the script edited");
        std::fs::write(pkg.join("helper.rs"), "// helper v2, edited\npub fn tag() -> &'static str {\n    \"a\"\n}\n").unwrap();
        check(&pkg, &log, 1, "a comment of the script's module edited");
        check(&pkg, &log, 1, "nothing changed after the replays: no run");
        // A new line moves the code under it, and with it the panic
        // locations in the binary: a run.
        std::fs::write(pkg.join("build.rs"), format!("// a new first line\n{edited}")).unwrap();
        check(&pkg, &log, 2, "a comment line added above code that can panic");
        std::fs::write(pkg.join("assets/a.txt"), "uno\n").unwrap();
        check(&pkg, &log, 3, "a file edited inside a rerun-if-changed dir");
        std::fs::write(pkg.join("assets/b.txt"), "two\n").unwrap();
        check(&pkg, &log, 4, "a file added inside a rerun-if-changed dir");
        std::fs::write(pkg.join("data.txt"), "v2\n").unwrap();
        check(&pkg, &log, 5, "a rerun-if-changed file edited");
        std::fs::write(pkg.join("helper.rs"), "// helper v2, edited\npub fn tag() -> &'static str {\n    \"b\"\n}\n").unwrap();
        check(&pkg, &log, 6, "a module of the script edited");
        let o = outputs(&pkg, &cache, Some(&env)).unwrap().unwrap();
        assert_eq!((o.rustc_cfg, o.rustc_env), (vec!["bs_b".to_string()], vec![("BS_DATA".to_string(), "v2".to_string())]));
        std::fs::write(pkg.join("Cargo.toml"), manifest.replace("edition", "description = \"x\"\nedition")).unwrap();
        check(&pkg, &log, 7, "the manifest edited");
        let out_dir = o.out_dir.unwrap();
        std::fs::remove_file(out_dir.join("gen.txt")).unwrap();
        check(&pkg, &log, 8, "a file gone from OUT_DIR");
        assert!(out_dir.join("gen.txt").exists(), "the run wrote it again");
        check(&pkg, &log, 8, "nothing changed again: no run");
        // A script with no rerun-if directive reruns on any package file
        // change (Cargo's default), and not on Cargo's own target dir.
        let (pkg2, log2) = (root.join("pkg2"), root.join("runs2.log"));
        std::fs::create_dir_all(pkg2.join("src")).unwrap();
        std::fs::write(pkg2.join("Cargo.toml"), manifest.replace("bsrerun", "bsdefault")).unwrap();
        std::fs::write(pkg2.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(
            pkg2.join("build.rs"),
            "fn main() {\n    use std::io::Write;\n    let log = std::env::var(\"LUST_BS_TEST_LOG2\").unwrap();\n    \
             std::fs::OpenOptions::new().create(true).append(true).open(log).unwrap().write_all(b\"run\\n\").unwrap();\n}\n",
        )
        .unwrap();
        let env2: Vec<(String, String)> =
            env.iter().cloned().chain([("LUST_BS_TEST_LOG2".to_string(), log2.to_string_lossy().into_owned())]).collect();
        let check2 = |want_runs: usize, what: &str| {
            let r = refresh(&pkg2, &cache, Some(&env2));
            if let Refresh::Failed { error, .. } = &r {
                panic!("{what}: the run failed: {error}");
            }
            assert_eq!(runs(&log2), want_runs, "{what}");
        };
        check2(1, "the first build runs the script");
        check2(1, "nothing changed (Cargo's target dir is not a package file): no run");
        std::fs::write(pkg2.join("src/main.rs"), "fn main() { }\n").unwrap();
        check2(2, "any package file edited, with no rerun-if directive");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// `build = false` and a custom `build = "gen.rs"` both win over the
    /// conventional build.rs.
    #[test]
    fn manifest_build_setting_wins() {
        let dir = std::env::temp_dir().join(format!("lust-buildscript-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("build.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\nbuild = false\n").unwrap();
        assert_eq!(build_script_for(&dir), None, "build = false");
        std::fs::write(dir.join("gen.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\nbuild = \"gen.rs\"\n").unwrap();
        assert_eq!(build_script_for(&dir), Some(dir.join("gen.rs")), "custom build path");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        assert_eq!(build_script_for(&dir), Some(dir.join("build.rs")), "conventional");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
