// Clang PGO for the release driver, per unit (rel-cache round 3).
//
// `LUST_PGO_GEN=1` builds an instrumented binary (`-fprofile-generate`; its runs write
// raw profiles where `LLVM_PROFILE_FILE` says). zig ships no profile runtime, so with a
// zig compiler that build uses `LUST_PGO_CC`, a clang of zig's LLVM major version, for
// the host (`gen_spec`; scripts/pgo-llvm.sh installs one). `LUST_PGO_USE=<file.profdata>`
// builds with a merged profile, which must come from the compiler's LLVM major. A profile must not change every object's cache key, so each
// unit's key holds only its own functions' records (`Mode::key_part`): a refreshed or
// pinned profile recompiles only the units whose own counts moved. Every unit with
// records compiles against the whole quantized profile (clang reads only the records of
// its module's fns). The thresholds LLVM derives from a profile's summary (hot, cold,
// the working-set size) are pinned to the whole profile's (`pins`, in every key),
// rounded to a power of 2 so a retrain keeps them, and size optimization keys on the
// pinned cold count only (`-pgso-cold-code-only`). `LUST_PGO_BITS=<n>` keeps n
// significant bits of every count (0 = exact), so a retrain on the same inputs moves
// fewer records (2 bits: 89% of tsgo's units keep their records). `LUST_PROFDATA` names
// the llvm-profdata of the compiler's LLVM.
//
// Record names: an external function is its own name; a static one is
// `<file>;<name>`, and every unit compiles under one file name (`STATIC_FILE`), so a
// static fn's record does not move when its unit's members do. A unit owns the records
// of the definitions it prints (`defined_externs`, `defined_statics`), and only those, so
// its key moves only with its own fns' records. A unit with `#include`s (the runtime, mimalloc) makes some
// fns with macros (`LUST_ABS(int8_t, i8)`), so its definitions come from its
// preprocessed text. A record no unit defines is of a fn this build does not have (a
// profile from an older state, or a `lust_*` fn only inline copies held) and goes to no
// unit.
//
// The fixed cost of a warm build with an unchanged profile (rel-llvm round 2026-10-19): it
// reads none of the profile's bytes (`profile_digest`: a stat record per profile), runs no
// llvm-profdata (`tool_version`), reads the records' index and not the profile text
// (`pgo-all.index`), and scans each unit's text once for its definitions
// (`definitions_of`). Its objects keep no call graph profile for the link
// (`drop_cg_profile`): lld would sort the link by it, for no run speed.
use super::CcSpec;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

/// The PGO mode of one release build.
pub enum Mode {
    Gen,
    Use(Pgo),
}

/// A loaded profile, split by unit.
pub struct Pgo {
    /// The digests (`Index`) of each unit's own record names, sorted, by the file name the
    /// unit compiles as (shared, not copied: a static fn of the unit header has a record in
    /// every unit).
    own: HashMap<String, Vec<u128>>,
    /// The flags that pin the summary thresholds to the whole profile's.
    pins: Vec<String>,
    records: usize,
    /// When every unit was split (`UnitDefs::all`): the records of macro-made fns (only a
    /// preprocessed text shows their definition), and the other records no unit defines (fns
    /// this build does not have: in no profile).
    counts: Option<(usize, usize)>,
    /// The quantized profile and pins came from the last build's work dir (`load`).
    kept: bool,
    /// The whole quantized profile (`pgo-all.profdata`) every unit with records compiles
    /// against, and its digest (the memo key).
    all: PathBuf,
    key: String,
}

impl Mode {
    /// The mode `LUST_PGO_GEN` / `LUST_PGO_USE` ask for, or None. `units` are (file name
    /// the unit compiles as, C text).
    /// `defs`: what each unit defines (`unit_defs`), with a profile to use.
    /// `warm`: a profile the streamed build loaded (`Warm`), whose index serves when it is this one.
    pub fn from_env(spec: &CcSpec, work: &Path, defs: Option<&UnitDefs>, warm: Option<&Warm>) -> Result<Option<Mode>, String> {
        if std::env::var_os("LUST_PGO_GEN").is_some_and(|v| v == "1") {
            return Ok(Some(Mode::Gen));
        }
        let Some(path) = std::env::var_os("LUST_PGO_USE") else { return Ok(None) };
        // The profile format and the fns' CFG hashes are those of one LLVM major.
        let cc = clang_major(&spec.cmd)?;
        let tool = tool_version(work)?;
        let pd = major_after(&tool, "LLVM version ")?;
        if pd != cc {
            return Err(format!("LUST_PROFDATA ({}) is LLVM {pd}, the compiler ({}) LLVM {cc}", profdata(), spec.cmd.join(" ")));
        }
        let bits = std::env::var("LUST_PGO_BITS").ok().and_then(|b| b.parse().ok()).unwrap_or(0);
        let defs = defs.ok_or("internal: a profile to use and no unit definitions")?;
        Pgo::load(Path::new(&path), bits, work, &tool, defs, warm).map(|p| Some(Mode::Use(p)))
    }

    /// Extra link flags: the profile runtime of an instrumented build.
    pub fn link_flags(&self) -> Vec<String> {
        match self {
            Mode::Gen => vec!["-fprofile-generate".into()],
            Mode::Use(_) => Vec::new(),
        }
    }

    /// The compile flags of the unit compiled as `file`; `fnless`: it defines no program fn
    /// (`fnless`). With a profile it first writes that unit's own profile into the work dir (a
    /// unit with no records gets none).
    pub fn unit_flags(&self, file: &str, fnless: bool) -> Result<Vec<String>, String> {
        match self {
            Mode::Gen => Ok(std::iter::once("-fprofile-generate".to_string()).chain(nopre_flags()).collect()),
            Mode::Use(p) => Ok(use_flags(&p.pins, p.own.get(file).is_some_and(|r| !r.is_empty()), &p.all, &p.key, fnless)),
        }
    }

    /// The profile part of the cache key of the unit compiled as `file` (`cc::unit_key`).
    pub fn key_part(&self, file: &str, fnless: bool) -> String {
        match self {
            Mode::Gen => format!("gen{} {STATIC_FILE}", if no_preinline() { " nopre" } else { "" }),
            Mode::Use(p) => use_key_part(&p.pins, p.own.get(file).map_or(&[][..], |v| &v[..]), fnless),
        }
    }

    /// What a unit's object needs after its compile, before the cache stores it: with a
    /// profile, its call graph profile out of the link (`cg_profile`).
    pub fn after_compile(&self, obj: &Path) -> Result<(), String> {
        match self {
            Mode::Use(_) if !cg_profile() => drop_cg_profile(obj),
            _ => Ok(()),
        }
    }

    /// For the build summary line.
    pub fn describe(&self) -> String {
        match self {
            Mode::Gen => "pgo instrumented".into(),
            Mode::Use(p) => {
                let kept = if p.kept { " (kept)" } else { "" };
                match p.counts {
                    Some((helpers, absent)) => format!("pgo {} records{kept}, {helpers} runtime helpers, {absent} of fns not in this build", p.records),
                    None => format!("pgo {} records{kept}", p.records),
                }
            }
        }
    }
}

fn nopre_flags() -> impl Iterator<Item = String> {
    no_preinline().then_some(["-mllvm".to_string(), "-disable-preinline".to_string()]).into_iter().flatten()
}

/// A use build's compile flags of a unit (`Mode::unit_flags`, `Warm::unit_flags`).
fn use_flags(pins: &[String], has_records: bool, all: &Path, key: &str, fnless: bool) -> Vec<String> {
    let mut flags = pins.to_vec();
    flags.extend(nopre_flags());
    // A unit with records reads the whole profile: clang takes only the records of
    // its module's fns, and the pins hold the summary to the whole profile's, so
    // the object is the one its own records would give, with no per-unit profile
    // to write and merge. Its key still holds only its own records (`key_part`).
    if has_records {
        flags.push(format!("-fprofile-use={}", all.display()));
        // A unit of glue (no program fn: a glue shard or a statics part) compiles the fns the
        // profile calls cold with no optimization: glue no training run reached costs clang
        // much time for little code (tsgo replay m9, m11: -7% of the missed units' compile
        // instructions, +0.9% tsc size). In fn units the same costs 2x the size of tsc.
        if fnless {
            flags.extend(["-mllvm".to_string(), "-pgo-cold-func-opt=optnone".to_string()]);
        }
        // zig cc caches a compile by its source and args, not by this file: the
        // profile's digest in the args keeps another profile at this path from
        // getting the object of the last one.
        flags.push(format!("-DLUST_PROFILE_DIGEST={key}"));
    }
    flags
}

/// A use build's key part of a unit with records `own` (`Mode::key_part`, `Warm::key_part`).
fn use_key_part(pins: &[String], own: &[u128], fnless: bool) -> String {
    let nopre = if no_preinline() { " nopre" } else { "" };
    let pins = pins.join(" ");
    let own: Vec<[u8; 16]> = own.iter().map(|d| d.to_le_bytes()).collect();
    // One part with `nopre`: with the call graph profile kept, keys are those of
    // builds before the flag.
    let flags = format!("{nopre}{}{}", if cg_profile() { "" } else { " nocg" }, if fnless && !own.is_empty() { " cold-optnone" } else { "" });
    let mut parts: Vec<&[u8]> = vec![pins.as_bytes(), flags.as_bytes(), STATIC_FILE.as_bytes()];
    parts.extend(own.iter().map(|b| b.as_slice()));
    digest(&parts)
}

/// A profile to use, loaded before a build's units exist (the streamed build, `stream.rs`):
/// `Pgo::load`'s warm path, its records' index kept (`pgo-all.index`'s text), so each unit's
/// own records are found when its text arrives (`own`), as `split` finds them for all units.
pub(super) struct Warm {
    text: String,
    /// Per index entry, sorted by record name: the name's start and length in `text`, its
    /// records' digest, and their number.
    spans: Vec<(usize, usize, u128, usize)>,
    pins: Vec<String>,
    all: PathBuf,
    key: String,
}

impl Warm {
    /// The LUST_PGO_USE profile, when this work dir holds its quantized profile, pins and
    /// index from an earlier build (else None: `build` loads it the cold way).
    pub(super) fn load(spec: &CcSpec, work: &Path) -> Result<Option<Warm>, String> {
        let Some(path) = std::env::var_os("LUST_PGO_USE") else { return Ok(None) };
        let cc = clang_major(&spec.cmd)?;
        let tool = tool_version(work)?;
        let pd = major_after(&tool, "LLVM version ")?;
        if pd != cc {
            return Err(format!("LUST_PROFDATA ({}) is LLVM {pd}, the compiler ({}) LLVM {cc}", profdata(), spec.cmd.join(" ")));
        }
        let bits: u32 = std::env::var("LUST_PGO_BITS").ok().and_then(|b| b.parse().ok()).unwrap_or(0);
        let (all, memo, idx) = (work.join("pgo-all.profdata"), work.join("pgo-all.memo"), work.join("pgo-all.index"));
        let key = digest(&[profile_digest(Path::new(&path), work)?.as_bytes(), &bits.to_le_bytes(), tool.as_bytes()]);
        let Some(pins) = std::fs::read_to_string(&memo)
            .ok()
            .and_then(|m| m.strip_prefix(&format!("{key}\n")).map(|p| p.split_whitespace().map(str::to_string).collect()))
        else {
            return Ok(None);
        };
        let Ok(text) = std::fs::read_to_string(&idx) else { return Ok(None) };
        if !all.exists() {
            return Ok(None);
        }
        Ok(Warm::of_index(text, pins, all, key))
    }

    /// A warm profile of an index file's `text` (`write_index`); None when it is not one.
    fn of_index(text: String, pins: Vec<String>, all: PathBuf, key: String) -> Option<Warm> {
        let base = text.as_ptr() as usize;
        let spans = read_index(&text)?.iter().map(|&(n, d, k)| (n.as_ptr() as usize - base, n.len(), d, k)).collect();
        Some(Warm { text, spans, pins, all, key })
    }

    fn name(&self, i: usize) -> &str {
        let (at, len, _, _) = self.spans[i];
        &self.text[at..at + len]
    }

    /// The index (`Index`), its names borrowed from this profile.
    fn index(&self) -> Index<'_> {
        (0..self.spans.len()).map(|i| (self.name(i), self.spans[i].2, self.spans[i].3)).collect()
    }

    /// The digests of the records the unit compiled as `file` owns, sorted, as `split` gives
    /// them: an external fn's for each of `exts`, a static fn's (`STATIC_FILE;<fn>`) for each
    /// of `statics`, and every static record of `file`'s own name.
    pub(super) fn own(&self, file: &str, exts: &[&str], statics: &[&str]) -> Vec<u128> {
        let n = self.spans.len();
        let find = |prefix: &str, name: &str| {
            let (mut lo, mut hi) = (0, n);
            while lo < hi {
                let mid = (lo + hi) / 2;
                match cmp_joined(self.name(mid), prefix, name) {
                    std::cmp::Ordering::Less => lo = mid + 1,
                    std::cmp::Ordering::Greater => hi = mid,
                    std::cmp::Ordering::Equal => return Some(mid),
                }
            }
            None
        };
        // `unit_defs` gives `split` each unit's externs sorted, without repeats.
        let mut ext: Vec<&str> = exts.to_vec();
        ext.sort_unstable();
        ext.dedup();
        let static_prefix = format!("{STATIC_FILE};");
        let mut ds: Vec<u128> = ext
            .iter()
            .filter_map(|e| find("", e))
            .chain(statics.iter().filter_map(|e| find(&static_prefix, e)))
            .map(|i| self.spans[i].2)
            .collect();
        if file != STATIC_FILE {
            let (mut lo, mut hi) = (0, n);
            while lo < hi {
                let mid = (lo + hi) / 2;
                if cmp_joined(self.name(mid), file, ";").is_lt() { lo = mid + 1 } else { hi = mid }
            }
            ds.extend(
                (lo..n).take_while(|&i| self.name(i).strip_prefix(file).is_some_and(|r| r.starts_with(';'))).map(|i| self.spans[i].2),
            );
        }
        ds.sort_unstable();
        ds
    }

    /// The compile flags of a unit with records `own` (`Mode::unit_flags`).
    pub(super) fn unit_flags(&self, own: &[u128], fnless: bool) -> Vec<String> {
        use_flags(&self.pins, !own.is_empty(), &self.all, &self.key, fnless)
    }

    /// The key part of a unit with records `own` (`Mode::key_part`).
    pub(super) fn key_part(&self, own: &[u128], fnless: bool) -> String {
        use_key_part(&self.pins, own, fnless)
    }
}

/// The compiler of an instrumented build for `spec`. zig ships no profile runtime, so for
/// zig `LUST_PGO_CC` names a clang of zig's LLVM major and that build targets the host,
/// where the training runs (the C is the release target's all the same). Any other
/// compiler must itself be a clang: the profiles are LLVM IR profiles.
pub fn gen_spec(spec: &CcSpec) -> Result<CcSpec, String> {
    let want = clang_major(&spec.cmd)?;
    if !super::is_zig(&spec.cmd[0]) {
        return Ok(spec.clone());
    }
    let words: Vec<String> = std::env::var("LUST_PGO_CC").unwrap_or_default().split_whitespace().map(str::to_string).collect();
    if words.is_empty() {
        return Err(format!(
            "an instrumented build (LUST_PGO_GEN=1) needs a profile runtime, and zig ships none: set LUST_PGO_CC to a clang of LLVM {want}, zig's version (scripts/pgo-llvm.sh installs one)"
        ));
    }
    let have = clang_major(&words)?;
    if have != want {
        return Err(format!("LUST_PGO_CC ({}) is LLVM {have}, zig cc LLVM {want}: its profile would not match", words.join(" ")));
    }
    Ok(CcSpec { triple: spec.triple.clone(), cxx: super::cxx_for(&words), cmd: words, statik: false, exe_suffix: spec.exe_suffix })
}

/// The LLVM major version of a clang command; an error for another compiler (gcc writes
/// its own profile format). It reads the `--version` line of `compiler_id`, which runs it
/// once a process (the unit keys hold it).
fn clang_major(cmd: &[String]) -> Result<u32, String> {
    let id = super::compiler_id(cmd);
    let first = id.split(" | ").nth(1).unwrap_or("");
    if first.is_empty() {
        return Err(format!("cannot run `{} --version`", cmd.join(" ")));
    }
    if !first.contains("clang version ") {
        return Err(format!("PGO needs clang (LLVM IR profiles); `{}` says: {first}", cmd.join(" ")));
    }
    major_after(first, "clang version ")
}

/// The major version number after `marker` in `text` (`clang version 20.1.2 ...`).
fn major_after(text: &str, marker: &str) -> Result<u32, String> {
    text.split(marker)
        .nth(1)
        .and_then(|r| r.split(['.', ' ']).next())
        .and_then(|m| m.trim().parse().ok())
        .ok_or_else(|| format!("no version after `{}` in: {}", marker.trim(), text.lines().next().unwrap_or("")))
}

/// The LLVM fns (`fl_<id>`, by id) whose largest counter in the `LUST_PGO_USE` profile is at
/// least `min`: the callees worth an inline copy (`cgen::ll::ll_inline_set`). None without a
/// profile to use. It follows the profile alone; `work/pgo-hot.memo` keeps it for the next
/// build of the same profile (a pinned profile: every merge).
pub fn hot_ids(work: &Path, min: u64) -> Result<Option<std::collections::HashSet<u64>>, String> {
    let Some(path) = std::env::var_os("LUST_PGO_USE") else { return Ok(None) };
    hot_ids_of(Path::new(&path), work, min, "pgo-hot.memo").map(Some)
}

/// The ids of the fns (`fl_<id>`, release C `f_<id>`) whose record in the profile at `path`
/// has a counter of at least `min`, memoized in `work` under `memo` by the profile's bytes
/// (`profile_digest`).
pub fn hot_ids_of(path: &Path, work: &Path, min: u64, memo: &str) -> Result<std::collections::HashSet<u64>, String> {
    let key = digest(&[b"hot", profile_digest(path, work)?.as_bytes(), &min.to_le_bytes()]);
    let memo = work.join(memo);
    if let Some(ids) = std::fs::read_to_string(&memo).ok().and_then(|m| m.strip_prefix(&format!("{key}\n")).map(str::to_string)) {
        return Ok(ids.lines().filter_map(|l| u64::from_str_radix(l, 16).ok()).collect());
    }
    let (_, records) = parse(&run(Command::new(profdata()).args(["merge", "--text", "-o", "-"]).arg(path))?);
    let mut ids: Vec<u64> = records
        .iter()
        .filter(|(_, rs)| rs.iter().any(|r| max_counter(r) >= min))
        .filter_map(|(name, _)| name.strip_prefix("fl_").or_else(|| name.strip_prefix("f_")).filter(|h| h.len() == 16).and_then(|h| u64::from_str_radix(h, 16).ok()))
        .collect();
    ids.sort_unstable();
    std::fs::create_dir_all(work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let text: String = ids.iter().map(|id| format!("{id:016x}\n")).collect();
    std::fs::write(&memo, format!("{key}\n{text}")).map_err(|e| format!("cannot write {}: {e}", memo.display()))?;
    Ok(ids.into_iter().collect())
}

/// The largest counter of one text-format record.
fn max_counter(r: &str) -> u64 {
    let mut lines = r.lines();
    let n = lines.by_ref().skip_while(|l| *l != "# Num Counters:").nth(1).and_then(|l| l.trim().parse::<usize>().ok()).unwrap_or(0);
    lines.skip_while(|l| *l != "# Counter Values:").skip(1).take(n).filter_map(|l| l.trim().parse::<u64>().ok()).max().unwrap_or(0)
}

fn profdata() -> String {
    std::env::var("LUST_PROFDATA").unwrap_or_else(|_| "llvm-profdata".into())
}

/// `llvm-profdata --version` of the `LUST_PROFDATA` tool: the profile's memo key holds it,
/// and its LLVM major must be the compiler's. A warm build runs no llvm-profdata (10-110 ms
/// a run, libLLVM's load from a cold page cache the most): `work/pgo-tool` keeps the answer
/// with the stamp (`file_stamp`) of the file PATH resolves the tool to.
fn tool_version(work: &Path) -> Result<String, String> {
    let tool = profdata();
    let file = super::resolve_bin(&tool);
    let stamp = file_stamp(&file).ok().flatten().filter(|(_, settled)| *settled).map(|(s, _)| format!("{} {s}", file.display()));
    let rec = work.join("pgo-tool");
    if let Some(s) = &stamp
        && let Some(v) = std::fs::read_to_string(&rec).ok().and_then(|t| t.strip_prefix(&format!("{s}\n")).map(str::to_string))
    {
        return Ok(v);
    }
    let v = run(Command::new(&tool).arg("--version"))?;
    if let Some(s) = stamp {
        write_record(work, &rec, &format!("{s}\n{v}"));
    }
    Ok(v)
}

/// Writes a record file of this work dir whole (a reader sees the old one or the new one);
/// a failed write only leaves the next build without it.
fn write_record(work: &Path, rec: &Path, text: &str) {
    let tmp = rec.with_extension(format!("tmp{}", std::process::id()));
    if std::fs::create_dir_all(work).is_ok() && std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, rec);
    }
}

fn run(cmd: &mut Command) -> Result<String, String> {
    let o = cmd.output().map_err(|e| format!("cannot run {:?}: {e}", cmd.get_program()))?;
    if !o.status.success() {
        return Err(format!("{:?} failed: {}", cmd.get_program(), String::from_utf8_lossy(&o.stderr)));
    }
    Ok(String::from_utf8_lossy(&o.stdout).into_owned())
}

/// The source file name every PGO unit compiles under (a C unit as `<work>/<unit>.d/lust.c`,
/// an LLVM unit by its IR's `source_filename`). PGO names an internal fn `<source name>;<fn>`;
/// the unit's own name hashes its members, so a merge that moved one fn would rename the
/// records of every static fn and helper in the unit. `split` owns these records by fn name.
pub(crate) const STATIC_FILE: &str = "lust.c";

/// Neither the instrumented nor the use build runs LLVM's PGO pre-inliner
/// (`LUST_PGO_PREINLINE=1` runs it). It inlines small callees before each fn's CFG hash is
/// taken, so a caller's record stops matching when a callee it inlined changes (a cross-unit
/// copy, a merge that edits the callee); without it a record follows its own fn only. tsgo, a
/// profile pinned at replay state 0: 99.4-99.8% of its counts still apply after 12 merges
/// (94.6-99.7% with it), at no measurable run-speed cost in either backend.
pub fn no_preinline() -> bool {
    std::env::var("LUST_PGO_PREINLINE").map_or(true, |v| v != "1")
}

/// The objects of a build with a profile keep no call graph profile for the link
/// (`drop_cg_profile`; `LUST_PGO_CGPROFILE=1` keeps it). clang writes one under
/// `-fprofile-use`, and lld sorts the link's sections by it: on a warm tsgo build that was
/// 180 ms of a 430 ms link, for no run speed (TypeORM wall 0.530 s sorted, 0.534 s not;
/// cycles 3-4% lower unsorted on TypeORM and the pgo-train set, within noise).
pub fn cg_profile() -> bool {
    std::env::var("LUST_PGO_CGPROFILE").is_ok_and(|v| v == "1")
}

/// ELF's section type of a call graph profile (`.llvm.call-graph-profile`).
const SHT_LLVM_CALL_GRAPH_PROFILE: u32 = 0x6fff_4c09;

/// Takes the call graph profile of the object `obj` out of its link: the section becomes a
/// plain one (SHT_PROGBITS), which its SHF_EXCLUDE flag still keeps out of the binary, so lld
/// has no call graph to sort the link by. Neither clang 20 nor zig cc has a flag that stops
/// the section or the sort. A binary from such objects is byte for byte the one from objects
/// with the section removed. An object that is not ELF64 little-endian, or has no such
/// section, stays as it is.
fn drop_cg_profile(obj: &Path) -> Result<(), String> {
    let mut b = std::fs::read(obj).map_err(|e| format!("cannot read {}: {e}", obj.display()))?;
    if b.len() < 64 || &b[..4] != b"\x7fELF" || b[4] != 2 || b[5] != 1 {
        return Ok(());
    }
    let u16_at = |b: &[u8], at: usize| u16::from_le_bytes([b[at], b[at + 1]]) as usize;
    let u64_at = |b: &[u8], at: usize| b.get(at..at + 8).map(|x| u64::from_le_bytes(x.try_into().unwrap()) as usize);
    let (Some(shoff), entsize) = (u64_at(&b, 0x28), u16_at(&b, 0x3a)) else { return Ok(()) };
    // More than 0xff00 sections: the count is section 0's sh_size.
    let count = match u16_at(&b, 0x3c) {
        0 => u64_at(&b, shoff + 0x20).unwrap_or(0),
        n => n,
    };
    let mut hit = false;
    for i in 0..count {
        let at = shoff + i * entsize + 4;
        let Some(t) = b.get(at..at + 4) else { break };
        if u32::from_le_bytes(t.try_into().unwrap()) == SHT_LLVM_CALL_GRAPH_PROFILE {
            b[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
            hit = true;
        }
    }
    if !hit {
        return Ok(());
    }
    // A new file: the compiler's output may be a link to its own cache's entry.
    let tmp = obj.with_extension("o.nocg");
    std::fs::write(&tmp, &b).and_then(|_| std::fs::rename(&tmp, obj)).map_err(|e| format!("cannot write {}: {e}", obj.display()))
}

fn digest(parts: &[&[u8]]) -> String {
    let (a, b) = crate::driver::cache::hash128(parts, 41);
    format!("{a:016x}{b:016x}")
}

/// The digest of the bytes of the profile at `path`, which every key and memo of a profile
/// holds. A build whose profile did not change reads none of it (tsgo's: 39-48 MB, from a
/// CI disk mounted cold): `work/pgo-stat-<path digest>` records the digest with the file's
/// size, inode, and modify and change times, and a file with the same four has the same
/// bytes. Every write moves the change time, but within one clock tick it can stay: a
/// record is made only for a file whose last change was over a second before it was read,
/// with the same four after the read.
pub(crate) fn profile_digest(path: &Path, work: &Path) -> Result<String, String> {
    let read_err = |e: std::io::Error| format!("cannot read {}: {e}", path.display());
    let stamp = file_stamp(path).map_err(read_err)?;
    let full = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let rec = work.join(format!("pgo-stat-{}", &digest(&[full.as_os_str().as_encoded_bytes()])[..16]));
    if let Some((s, _)) = &stamp
        && let Some(d) = std::fs::read_to_string(&rec).ok().and_then(|t| t.strip_prefix(&format!("{s} ")).map(|d| d.trim_end().to_string()))
    {
        return Ok(d);
    }
    let d = digest(&[&std::fs::read(path).map_err(read_err)?]);
    if let Some((s, settled)) = stamp
        && settled
        && file_stamp(path).is_ok_and(|after| after.is_some_and(|(a, _)| a == s))
    {
        write_record(work, &rec, &format!("{s} {d}\n"));
    }
    Ok(d)
}

/// What `profile_digest` compares (size, inode, and modify and change times in ns), and
/// whether the file's last change is over a second old. None where the platform has no
/// inode or change time: the profile is read every build.
#[cfg(unix)]
fn file_stamp(path: &Path) -> std::io::Result<Option<(String, bool)>> {
    use std::os::unix::fs::MetadataExt;
    let m = std::fs::metadata(path)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let s = format!("{} {} {}.{:09} {}.{:09}", m.len(), m.ino(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec());
    Ok(Some((s, now - m.ctime() >= 2)))
}
#[cfg(not(unix))]
fn file_stamp(path: &Path) -> std::io::Result<Option<(String, bool)>> {
    std::fs::metadata(path).map(|_| None)
}

impl Pgo {
    /// Reads `path` (an indexed profile), quantizes it to `bits`, pins the summary
    /// thresholds of the whole quantized profile, and splits the records by unit. `tool` is
    /// the `llvm-profdata --version` text.
    fn load(path: &Path, bits: u32, work: &Path, tool: &str, defs: &UnitDefs, warm: Option<&Warm>) -> Result<Pgo, String> {
        std::fs::create_dir_all(work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
        let (txt, all, memo) = (work.join("pgo-all.proftext"), work.join("pgo-all.profdata"), work.join("pgo-all.memo"));
        // The quantized profile and its pins depend only on the profile, `bits` and the
        // tool: a warm build of the same ones (a pinned profile, every merge) reads them
        // back from the last build's work dir instead of running llvm-profdata three times
        // (~2.8 s for tsgo's), and the records' index (`Index`) instead of the profile text.
        // `memo` holds their digest and the pins, and is written last.
        let idx = work.join("pgo-all.index");
        let key = digest(&[profile_digest(path, work)?.as_bytes(), &bits.to_le_bytes(), tool.as_bytes()]);
        // The streamed build loaded this profile's index already (`Warm`): no second read.
        if let Some(w) = warm.filter(|w| w.key == key && all.exists()) {
            let index = w.index();
            let (own, counts) = split(&index, defs);
            return Ok(Pgo { own, pins: w.pins.clone(), records: index.iter().map(|(_, _, n)| n).sum(), counts, kept: true, all, key });
        }
        let pins_kept = std::fs::read_to_string(&memo)
            .ok()
            .and_then(|m| m.strip_prefix(&format!("{key}\n")).map(|pins| pins.split_whitespace().map(str::to_string).collect::<Vec<_>>()));
        let index_text = pins_kept.as_ref().and_then(|_| std::fs::read_to_string(&idx).ok());
        let records: Records;
        let (index, pins, hit) = match (pins_kept, index_text.as_deref().and_then(read_index)) {
            (Some(pins), Some(index)) => (index, pins, true),
            _ => {
                let _ = std::fs::remove_file(&memo);
                let text = run(Command::new(profdata()).args(["merge", "--text", "-o", "-"]).arg(path))?;
                let (head, mut recs) = parse(&text);
                if bits > 0 {
                    for r in recs.values_mut().flatten() {
                        *r = Arc::from(quantize(r, bits));
                    }
                }
                records = recs;
                let mut names: Vec<&String> = records.keys().collect();
                names.sort();
                let body: Vec<&str> = names.iter().flat_map(|n| records[*n].iter().map(|r| &**r)).collect();
                std::fs::write(&txt, format!("{head}{}\n\n", body.join("\n\n"))).map_err(|e| format!("cannot write {}: {e}", txt.display()))?;
                run(Command::new(profdata()).arg("merge").arg("-o").arg(&all).arg(&txt))?;
                let pins = pins(&run(Command::new(profdata()).args(["show", "--detailed-summary"]).arg(&all))?)?;
                let index = index(&records);
                write_index(&idx, &index)?;
                std::fs::write(&memo, format!("{key}\n{}\n", pins.join(" "))).map_err(|e| format!("cannot write {}: {e}", memo.display()))?;
                (index, pins, false)
            }
        };
        if hit && !all.exists() {
            run(Command::new(profdata()).arg("merge").arg("-o").arg(&all).arg(&txt))?;
        }
        let (own, counts) = split(&index, defs);
        Ok(Pgo { own, pins, records: index.iter().map(|(_, _, n)| n).sum(), counts, kept: hit, all, key })
    }
}

/// The external fns each unit defines, by the file name it compiles as, the names only a
/// preprocessed text shows (macro-made), and the static fns each unit defines. A unit's own
/// records follow from its own definitions alone (`split`), so a build gives only the units it
/// compiles; `all`: these are all of the build's units (the summary's counts need them all).
pub(super) struct UnitDefs {
    units: Vec<(String, Vec<String>)>,
    macro_made: std::collections::HashSet<String>,
    statics: Vec<(String, Vec<String>)>,
    all: bool,
}

/// The definitions of each unit, on every thread: those of its text, scanned once
/// (`definitions_of`, which its PGO file name reads too), or of its preprocessed text when
/// the caller has one (a unit with `#include`s; `cc::preprocessed`), so the fns its macros
/// make count as its own. `all`: as `UnitDefs::all`.
pub(super) fn unit_defs(units: &[(String, &Defs, Option<&[u8]>)], all: bool) -> UnitDefs {
    let owned = |v: &[&str]| v.iter().map(|n| n.to_string()).collect::<Vec<String>>();
    let per: Vec<(String, Vec<String>, Vec<String>, Vec<String>)> = crate::pool::par_map(units, |(file, raw, pre)| match pre {
        None => {
            let mut names = owned(&raw.0);
            names.sort_unstable();
            names.dedup();
            (file.clone(), names, Vec::new(), owned(&raw.1))
        }
        Some(pre) => {
            let pre = String::from_utf8_lossy(pre);
            let (names, statics) = definitions_of(&pre);
            let raw: std::collections::HashSet<&str> = raw.0.iter().copied().collect();
            let made = names.iter().filter(|n| !raw.contains(*n)).map(|n| n.to_string()).collect();
            (file.clone(), owned(&names), made, owned(&statics))
        }
    });
    let mut macro_made = std::collections::HashSet::new();
    let mut units = Vec::with_capacity(per.len());
    let mut statics = Vec::with_capacity(per.len());
    for (file, names, made, st) in per {
        macro_made.extend(made);
        statics.push((file.clone(), st));
        units.push((file, names));
    }
    UnitDefs { units, macro_made, statics, all }
}

/// The digests (`Index`) of each unit's own record names, by the file name it compiles as,
/// sorted; with all units (`UnitDefs::all`), how many of those names are of macro-made fns
/// and how many external ones no unit defines. An external fn's record goes to every unit that
/// defines it, a static fn's (`STATIC_FILE;<fn>`) to every unit that defines a static fn of
/// that name, a static record of another file to the unit of that file: a unit's records
/// depend on its own definitions only. Each unit finds its names in the sorted index by
/// binary search, on every thread, with no map of the index built first (tsgo's LLVM build:
/// 728k records over ~4,300 units, each with the unit header's static fns).
fn split(index: &[(&str, u128, usize)], defs: &UnitDefs) -> (HashMap<String, Vec<u128>>, Option<(usize, usize)>) {
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
    debug_assert!(index.windows(2).all(|w| w[0].0 < w[1].0), "the index is sorted by name");
    let find = |prefix: &str, name: &str| index.binary_search_by(|e| cmp_joined(e.0, prefix, name)).ok();
    let mut files: HashMap<&str, (&[String], &[String])> = HashMap::new();
    for (f, ns) in &defs.units {
        files.entry(f.as_str()).or_default().0 = ns;
    }
    for (f, ns) in &defs.statics {
        files.entry(f.as_str()).or_default().1 = ns;
    }
    let files: Vec<(&str, &[String], &[String])> = files.into_iter().map(|(f, (e, s))| (f, e, s)).collect();
    let claimed: Vec<AtomicBool> = index.iter().map(|_| AtomicBool::new(false)).collect();
    let static_prefix = format!("{STATIC_FILE};");
    let own = crate::pool::par_map(&files, |&(f, exts, statics)| {
        let mut ds = Vec::with_capacity(exts.len() + statics.len());
        let mut take = |i: usize| {
            ds.push(index[i].1);
            claimed[i].store(true, Relaxed);
        };
        exts.iter().filter_map(|n| find("", n)).for_each(&mut take);
        statics.iter().filter_map(|n| find(&static_prefix, n)).for_each(&mut take);
        // The static records of this file's own name: one run of the sorted index.
        if f != STATIC_FILE {
            let lo = index.partition_point(|e| cmp_joined(e.0, f, ";").is_lt());
            (lo..index.len()).take_while(|&i| index[i].0.strip_prefix(f).is_some_and(|r| r.starts_with(';'))).for_each(&mut take);
        }
        ds.sort_unstable();
        (f.to_string(), ds)
    });
    let counts = defs.all.then(|| {
        let (mut helpers, mut absent) = (0, 0);
        for ((n, _, _), c) in index.iter().zip(&claimed) {
            if !n.contains(';') {
                match c.load(Relaxed) {
                    true => helpers += defs.macro_made.contains(*n) as usize,
                    false => absent += 1,
                }
            }
        }
        (helpers, absent)
    });
    (own.into_iter().filter(|(_, d)| !d.is_empty()).collect(), counts)
}

/// `a` against `prefix` followed by `name`, in byte order, with no joined copy made.
fn cmp_joined(a: &str, prefix: &str, name: &str) -> std::cmp::Ordering {
    let (a, p) = (a.as_bytes(), prefix.as_bytes());
    let k = a.len().min(p.len());
    match a[..k].cmp(&p[..k]) {
        std::cmp::Ordering::Equal if a.len() < p.len() => std::cmp::Ordering::Less,
        std::cmp::Ordering::Equal => a[p.len()..].cmp(name.as_bytes()),
        o => o,
    }
}

/// A text profile's records by fn name: every record of the name (static fns of one source
/// name can differ in body, so in hash, across units).
type Records = HashMap<String, Vec<Arc<str>>>;

/// Per record name, sorted by name, the digest of its records (sorted) and their number: all
/// a unit's key reads of the profile (`key_part`), so a warm build of the same profile reads
/// this file (`pgo-all.index`) and not the profile text (tsgo's LLVM profile: 731k records).
/// The names borrow the records' keys or the index file's text.
type Index<'a> = Vec<(&'a str, u128, usize)>;

fn index(records: &Records) -> Index<'_> {
    let mut index: Index = records
        .iter()
        .map(|(n, rs)| {
            let mut rs: Vec<&[u8]> = rs.iter().map(|r| r.as_bytes()).collect();
            rs.sort_unstable();
            let (a, b) = crate::driver::cache::hash128(&rs, 41);
            (n.as_str(), ((a as u128) << 64) | b as u128, rs.len())
        })
        .collect();
    index.sort_unstable_by(|a, b| a.0.cmp(b.0));
    index
}

fn write_index(path: &Path, index: &Index) -> Result<(), String> {
    let mut text = String::with_capacity(index.len() * 64);
    for (n, d, k) in index {
        let _ = writeln!(text, "{n}\t{d:032x}\t{k}");
    }
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// The index in `text` (an index file's), names borrowed from it; None when it is not an
/// index or not sorted: the caller reads the profile again. Its lines are parsed on every
/// thread, in chunks.
fn read_index(text: &str) -> Option<Index<'_>> {
    let mut chunks = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let cut = rest.get(1 << 20..).and_then(|r| r.find('\n')).map_or(rest.len(), |k| (1 << 20) + k + 1);
        let (chunk, tail) = rest.split_at(cut);
        chunks.push(chunk);
        rest = tail;
    }
    fn parse(line: &str) -> Option<(&str, u128, usize)> {
        let mut f = line.rsplitn(3, '\t');
        let (k, d, n) = (f.next()?.parse().ok()?, u128::from_str_radix(f.next()?, 16).ok()?, f.next()?);
        Some((n, d, k))
    }
    let parts = crate::pool::par_map(&chunks, |c| c.lines().map(parse).collect::<Option<Vec<_>>>());
    let mut index = Vec::with_capacity(text.len() / 64);
    for p in parts {
        index.extend(p?);
    }
    index.windows(2).all(|w| w[0].0 < w[1].0).then_some(index)
}

/// An IR PGO text profile: the header (flag lines) and the records by name.
fn parse(text: &str) -> (String, Records) {
    let mut head = String::new();
    let mut lines = text.lines().peekable();
    while let Some(l) = lines.peek() {
        if (l.starts_with('#') || l.starts_with(':')) && *l != "# Func Hash:" {
            head.push_str(l);
            head.push('\n');
            lines.next();
        } else {
            break;
        }
    }
    // A record starts with its name, which never begins with `#` or `:`.
    let mut records: Records = HashMap::new();
    let mut cur: Vec<&str> = Vec::new();
    for l in lines.chain(std::iter::once("")) {
        if l.is_empty() {
            if let Some(name) = cur.first() {
                records.entry(name.to_string()).or_default().push(Arc::from(cur.join("\n")));
            }
            cur.clear();
        } else {
            cur.push(l);
        }
    }
    (head, records)
}

/// `r` with every counter and value-site count cut to `bits` significant bits.
fn quantize(r: &str, bits: u32) -> String {
    let q = |v: u64| if v < (1 << bits) { v } else { let s = 64 - v.leading_zeros() - bits; (v >> s) << s };
    let mut mode = "";
    let mut out: Vec<String> = Vec::new();
    for l in r.lines() {
        if l.starts_with('#') {
            mode = l;
            out.push(l.into());
        } else if mode == "# Counter Values:" && let Ok(v) = l.parse::<u64>() {
            out.push(q(v).to_string());
        } else if mode.contains("Value") && let Some((k, c)) = l.rsplit_once(':') && let Ok(v) = c.parse::<u64>() {
            out.push(format!("{k}:{}", q(v)));
        } else {
            out.push(l.into());
        }
    }
    out.join("\n")
}

/// The `-mllvm` flags that pin LLVM's summary thresholds (ProfileSummaryInfo) to those of
/// a whole profile (`llvm-profdata show --detailed-summary`): the hot and cold counts
/// (the 99% and 99.9999% cutoffs' minimum counts, rounded down to a power of 2), size
/// optimization by the cold count only, and the working-set flags (the hot cutoff's block
/// count against 12500 and 15000).
fn pins(show: &str) -> Result<Vec<String>, String> {
    let (mut hot, mut cold, mut blocks) = (None, None, 0u64);
    for l in show.lines() {
        // LLVM 18: "<n> blocks with count >= <min> account for <p> percentage of the total counts."
        // LLVM 20: "<n> blocks (<f>%) with count >= <min> account for <p>% of the total counts."
        let w: Vec<&str> = l.split_whitespace().collect();
        let at = |s: &str| w.iter().position(|x| *x == s);
        let (Some(ge), Some(f)) = (at(">="), at("for")) else { continue };
        if w.get(1) != Some(&"blocks") {
            continue;
        }
        let n = w[0].parse::<u64>().unwrap_or(0);
        let min = w.get(ge + 1).and_then(|m| m.parse::<u64>().ok()).unwrap_or(0);
        match w.get(f + 1).map(|p| p.trim_end_matches('%')) {
            Some("99") => (hot, blocks) = (Some(min), n),
            Some("99.9999") => cold = Some(min),
            _ => {}
        }
    }
    let (Some(hot), Some(cold)) = (hot, cold) else { return Err(format!("no detailed summary in llvm-profdata show output:\n{show}")) };
    let pow2 = |v: u64| if v == 0 { 0 } else { 1u64 << (63 - v.leading_zeros()) };
    let ws = |limit: u64| if blocks > limit { 0 } else { u32::MAX };
    Ok([
        format!("-profile-summary-hot-count={}", pow2(hot)),
        format!("-profile-summary-cold-count={}", pow2(cold)),
        "-pgso-cold-code-only=true".into(),
        format!("-profile-summary-huge-working-set-size-threshold={}", ws(15000)),
        format!("-profile-summary-large-working-set-size-threshold={}", ws(12500)),
    ]
    .into_iter()
    .flat_map(|f| ["-mllvm".to_string(), f])
    .collect())
}

/// The external functions unit `c` defines: an identifier that starts a column-0
/// declarator line and whose parameter list (over any number of lines) is followed by
/// a body. A prototype ends in `;` instead; a static function's record is owned by name
/// (`defined_statics`), so it is not listed.
pub fn defined_externs(c: &str) -> Vec<&str> {
    definitions_of(c).0
}

/// The static functions `c` defines, as `defined_externs` finds the external ones.
pub fn defined_statics(c: &str) -> Vec<&str> {
    definitions_of(c).1
}

/// What a unit's text defines: its external fns (`defined_externs`), then its static fns.
pub(super) type Defs<'a> = (Vec<&'a str>, Vec<&'a str>);

/// True when a unit's definitions (`definitions_of`) hold no fn of the program, external or
/// static, only glue, helpers and statics: release C's glue shards and statics parts. A
/// program fn is `f_<id>` (release C, 16 hex digits) or `f<slot>` (numbered C).
pub(super) fn fnless(defs: &Defs) -> bool {
    let program = |n: &&str| match n.strip_prefix('f') {
        Some(r) => match r.strip_prefix('_') {
            Some(id) => id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit()),
            None => !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()),
        },
        None => false,
    };
    !defs.0.iter().chain(&defs.1).any(program)
}

/// The external and the static functions `c` defines, in one pass over its lines (a warm
/// tsgo build scans ~1 GB of unit text: its PGO file names and the split both read this).
pub(super) fn definitions_of(c: &str) -> Defs<'_> {
    let b = c.as_bytes();
    let (mut ext, mut stat) = (Vec::new(), Vec::new());
    let mut at = 0;
    while at < b.len() {
        let end = c[at..].find('\n').map_or(b.len(), |k| at + k);
        let line = &c[at..end];
        let first = line.bytes().next().unwrap_or(b' ');
        if first.is_ascii_alphabetic() || first == b'_' {
            let is_static = line.starts_with("static ");
            let skip = ["static", "typedef", "struct", "union", "enum", "extern", "#"];
            if (is_static || !skip.iter().any(|s| line.starts_with(s)))
                && let Some((name, open)) = declarator_name(line)
            {
                let mut depth = 0usize;
                let mut i = at + open;
                while i < b.len() {
                    match b[i] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                if c.get(i + 1..).is_some_and(|rest| rest.trim_start().starts_with('{')) {
                    if is_static { stat.push(name) } else { ext.push(name) }
                }
            }
        }
        at = end + 1;
    }
    (ext, stat)
}

/// The first identifier on `line` that is followed by `(` and is not an attribute or a
/// type operator, and the byte index of that `(`.
fn declarator_name(line: &str) -> Option<(&str, usize)> {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let s = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let id = &line[s..i];
            let mut j = i;
            while j < b.len() && b[j] == b' ' {
                j += 1;
            }
            if j < b.len() && b[j] == b'(' {
                if matches!(id, "__attribute__" | "__declspec" | "_Alignas" | "__typeof__" | "typeof" | "__asm__") {
                    // Skip the attribute's own parenthesized list.
                    let mut depth = 0usize;
                    while j < b.len() {
                        match b[j] {
                            b'(' => depth += 1,
                            b')' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                    i = j + 1;
                    continue;
                }
                return Some((id, j));
            }
        } else {
            i += 1;
        }
    }
    None
}

/// A content-derived file name for a unit (the record names of its static functions
/// carry it): a hash of the external functions it defines (`defined_externs`), so it stays
/// while its member set does. None for a unit that defines none.
pub(super) fn stable_name(externs: &[&str]) -> Option<String> {
    if externs.is_empty() {
        return None;
    }
    let mut ext = externs.to_vec();
    ext.sort_unstable();
    let (a, _) = crate::driver::cache::hash128(&[ext.join(",").as_bytes()], 43);
    Some(format!("x{a:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A profile's digest is that of its bytes. A file just written gets no stat record
    /// (it could change again in the same clock tick); a record whose stat matches is taken
    /// without reading the file, and a changed file no longer matches it.
    #[cfg(unix)]
    #[test]
    fn a_profile_digest_reads_the_file_only_when_its_stat_moved() {
        let dir = std::env::temp_dir().join(format!("lust-pgo-stat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("use.profdata");
        std::fs::write(&p, b"profile one").unwrap();
        assert_eq!(profile_digest(&p, &dir).unwrap(), digest(&[b"profile one"]));
        let stat_file = |e: &std::fs::DirEntry| e.file_name().to_string_lossy().starts_with("pgo-stat-");
        let records = std::fs::read_dir(&dir).unwrap().flatten().filter(stat_file).count();
        assert_eq!(records, 0, "a file changed within the last second gets no record");
        let full = std::fs::canonicalize(&p).unwrap();
        let name = format!("pgo-stat-{}", &digest(&[full.as_os_str().as_encoded_bytes()])[..16]);
        let (stamp, _) = file_stamp(&p).unwrap().unwrap();
        std::fs::write(dir.join(&name), format!("{stamp} recorded\n")).unwrap();
        assert_eq!(profile_digest(&p, &dir).unwrap(), "recorded", "a matching stat is not read");
        std::fs::write(&p, b"profile two!").unwrap();
        assert_eq!(profile_digest(&p, &dir).unwrap(), digest(&[b"profile two!"]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An ELF object's call graph profile section becomes a plain section (lld then has no
    /// call graph to sort by); its other sections and a file that is not ELF stay as they are.
    #[test]
    fn a_call_graph_profile_section_is_retyped() {
        let mut elf = vec![0u8; 64 + 3 * 64];
        elf[..6].copy_from_slice(b"\x7fELF\x02\x01");
        elf[0x28..0x30].copy_from_slice(&64u64.to_le_bytes());
        elf[0x3a..0x3c].copy_from_slice(&64u16.to_le_bytes());
        elf[0x3c..0x3e].copy_from_slice(&3u16.to_le_bytes());
        let ty = |e: &[u8], i: usize| u32::from_le_bytes(e[64 + i * 64 + 4..64 + i * 64 + 8].try_into().unwrap());
        elf[64 + 64 + 4..64 + 64 + 8].copy_from_slice(&SHT_LLVM_CALL_GRAPH_PROFILE.to_le_bytes());
        elf[64 + 128 + 4..64 + 128 + 8].copy_from_slice(&9u32.to_le_bytes());
        let dir = std::env::temp_dir().join(format!("lust-pgo-cg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (o, other) = (dir.join("u.o"), dir.join("v.o"));
        std::fs::write(&o, &elf).unwrap();
        std::fs::write(&other, b"not an object").unwrap();
        drop_cg_profile(&o).unwrap();
        drop_cg_profile(&other).unwrap();
        let after = std::fs::read(&o).unwrap();
        assert_eq!((ty(&after, 0), ty(&after, 1), ty(&after, 2)), (0, 1, 9));
        assert_eq!(after.len(), elf.len());
        assert_eq!(std::fs::read(&other).unwrap(), b"not an object");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn defined_externs_finds_definitions_not_prototypes() {
        let c = "int f(int a);\nint f(int a) {\n  return a;\n}\nstatic int g(void) { return 1; }\nRcI_A* tramp_1(O_A p1) { return 0; }\n__attribute__((noreturn)) void lust_die(const char *m,\n    int n)\n{\n}\nstruct S { int (*cb)(void); };\n  int h(void) {\n";
        assert_eq!(defined_externs(c), vec!["f", "tramp_1", "lust_die"]);
    }

    fn rec(n: &str, c: u64) -> String {
        format!("{n}\n# Func Hash:\n7\n# Num Counters:\n1\n# Counter Values:\n{c}")
    }

    /// A loaded profile of `records` (name, count) over units that define `defs` (file,
    /// fn names; `macro_made` of them only a preprocessed text shows).
    fn split_mode(records: &[(&str, u64)], defs: &[(&str, &[&str])], macro_made: &[&str]) -> Mode {
        let recs: Records = records.iter().map(|(n, c)| (n.to_string(), vec![Arc::from(rec(n, *c))])).collect();
        let defs = UnitDefs {
            units: defs.iter().map(|(f, ns)| (f.to_string(), ns.iter().map(|n| n.to_string()).collect())).collect(),
            macro_made: macro_made.iter().map(|n| n.to_string()).collect(),
            statics: Vec::new(),
            all: true,
        };
        let (own, counts) = split(&index(&recs), &defs);
        Mode::Use(Pgo { own, pins: vec!["-mllvm".into(), "-x".into()], records: recs.len(), counts, kept: false, all: PathBuf::new(), key: String::new() })
    }

    /// Two static fns of one source name with different bodies are two records of one name:
    /// the profile keeps both (clang picks the one whose CFG hash matches).
    #[test]
    fn a_record_s_largest_counter_is_read() {
        let r = "fl_00000000000000aa\n# Func Hash:\n12\n# Num Counters:\n3\n# Counter Values:\n5\n70000\n9\n# Num Value Kinds:\n0";
        assert_eq!(super::max_counter(r), 70000);
        assert_eq!(super::max_counter("f\n# Func Hash:\n1\n# Num Counters:\n1\n# Counter Values:\n0"), 0);
    }

    #[test]
    fn a_name_keeps_every_record_it_has() {
        let text = format!(":ir\n{}\n\n{}\n", rec("lust.c;h", 3), rec("lust.c;h", 4).replace("\n7\n", "\n8\n"));
        let (_, recs) = parse(&text);
        assert_eq!(recs["lust.c;h"].len(), 2);
    }

    /// The index a warm build reads back gives each name the digest and count it was written
    /// with; a text that is not a sorted index is no index (the profile is read again).
    #[test]
    fn an_index_reads_back_as_written() {
        let text = format!(":ir\n{}\n\n{}\n\n{}\n", rec("f", 3), rec("lust.c;h", 4), rec("lust.c;h", 5).replace("\n7\n", "\n8\n"));
        let (_, recs) = parse(&text);
        let ix = index(&recs);
        let p = std::env::temp_dir().join(format!("lust-pgo-index-{}", std::process::id()));
        write_index(&p, &ix).unwrap();
        let file = std::fs::read_to_string(&p).unwrap();
        let _ = std::fs::remove_file(&p);
        assert_eq!(read_index(&file).as_ref(), Some(&ix));
        assert_eq!(ix.iter().find(|e| e.0 == "lust.c;h").map(|e| e.2), Some(2));
        let unsorted: String = file.lines().rev().map(|l| format!("{l}\n")).collect();
        assert!(read_index(&unsorted).is_none());
        assert!(read_index("not an index\n").is_none());
    }

    /// Only a unit of glue with records compiles its cold fns as optnone, and only its key
    /// says so: a fn unit's flags and key stay as they were.
    #[test]
    fn a_unit_of_glue_compiles_its_cold_fns_as_optnone() {
        let glue = "void drop_A_(A *p) {\n}\nstatic void dgc_A_(A *p) {\n}\n";
        let fns = "void drop_A_(A *p) {\n}\nint f_0123456789abcdef(int a) {\n}\n";
        let numbered = "static int f12(int a) {\n}\n";
        assert!(fnless(&definitions_of(glue)));
        assert!(!fnless(&definitions_of(fns)), "a release C fn");
        assert!(!fnless(&definitions_of(numbered)), "a numbered C fn");
        assert!(fnless(&definitions_of("int fmt_x(int a) {\n}\nint f_12(int a) {\n}\n")), "not program fn names");
        let pins = vec!["-mllvm".to_string(), "-x".to_string()];
        let cold = |f: &[String]| f.windows(2).any(|w| w[0] == "-mllvm" && w[1] == "-pgo-cold-func-opt=optnone");
        assert!(cold(&use_flags(&pins, true, Path::new("p"), "k", true)));
        assert!(!cold(&use_flags(&pins, true, Path::new("p"), "k", false)));
        assert!(!cold(&use_flags(&pins, false, Path::new("p"), "k", true)), "no profile, no cold fns");
        assert_ne!(use_key_part(&pins, &[1], true), use_key_part(&pins, &[1], false));
        assert_eq!(use_key_part(&pins, &[], true), use_key_part(&pins, &[], false), "a unit with no records keeps its key");
    }

    /// A static fn's record (`STATIC_FILE;<fn>`) goes to every unit that defines a static fn
    /// of that name, whatever the unit is called: a merge that renames a unit keeps it.
    #[test]
    fn a_static_record_follows_its_fn_not_its_unit() {
        let recs: Records =
            [("lust.c;lh_aa_0", 5), ("lust.c;lust_mix", 2), ("x1.c;old", 3)].iter().map(|(n, c)| (n.to_string(), vec![Arc::from(rec(n, *c))])).collect();
        let st = |f: &str, ns: &[&str]| (f.to_string(), ns.iter().map(|n| n.to_string()).collect::<Vec<_>>());
        let defs = UnitDefs {
            units: Vec::new(),
            macro_made: Default::default(),
            statics: vec![st("x9.ll", &["lh_aa_0", "lust_mix"]), st("x7.c", &["lust_mix"]), st("x1.c", &[]), st("x1.ca", &[])],
            all: true,
        };
        let ix = index(&recs);
        let digest_of = |n: &str| ix.iter().find(|e| e.0 == n).map(|e| e.1).unwrap();
        let (own, counts) = split(&ix, &defs);
        assert_eq!(own["x9.ll"].len(), 2);
        assert_eq!(own["x7.c"], vec![digest_of("lust.c;lust_mix")]);
        assert_eq!(own["x1.c"], vec![digest_of("x1.c;old")], "another file's static record stays with that file");
        assert!(!own.contains_key("x1.ca"), "a file whose name starts with another's takes none of its records");
        assert_eq!(counts, Some((0, 0)));
        // A unit split alone (a warm build splits only the units it compiles) gets the same
        // records; the counts need every unit.
        let alone = UnitDefs { units: Vec::new(), macro_made: Default::default(), statics: vec![st("x7.c", &["lust_mix"])], all: false };
        let (own7, counts) = split(&ix, &alone);
        assert_eq!((own7.get("x7.c"), counts), (own.get("x7.c"), None));
        assert_eq!(defined_statics("static inline int lust_mix(int a) {\n}\nstatic int g;\nint f(void) {\n}\n"), vec!["lust_mix"]);
    }

    /// A warm profile (`Warm::own`, the streamed build's) gives each unit the records `split`
    /// gives it: externs repeated or unsorted, statics repeated, a file's own static records.
    #[test]
    fn a_warm_profile_finds_each_units_records_as_split_does() {
        let mut records: Records = HashMap::new();
        for (n, r) in [
            ("f_a", "1"),
            ("f_b", "2"),
            ("f_c", "3"),
            ("lust.c;s1", "4"),
            ("lust.c;s2", "5"),
            ("x9.c;t", "6"),
            ("x9.c;u", "7"),
            ("x90.c;v", "8"),
            ("f_a", "9"),
        ] {
            records.entry(n.to_string()).or_default().push(Arc::from(rec(n, r.parse().unwrap()).as_str()));
        }
        let ix = index(&records);
        let dir = std::env::temp_dir().join(format!("lust-pgo-warm-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        write_index(&dir.join("i"), &ix).unwrap();
        let w = Warm::of_index(std::fs::read_to_string(dir.join("i")).unwrap(), Vec::new(), PathBuf::new(), String::new()).unwrap();
        let units: [(&str, Vec<&str>, Vec<&str>); 3] = [
            ("x9.c", vec!["f_b", "f_a", "f_a"], vec!["s1", "s1"]),
            ("x90.c", vec!["f_c", "f_none"], vec!["s2", "s3"]),
            ("lust.c", vec![], vec!["s1"]),
        ];
        let defs = UnitDefs {
            units: units
                .iter()
                .map(|(f, e, _)| {
                    (f.to_string(), {
                        let mut e: Vec<String> = e.iter().map(|s| s.to_string()).collect();
                        e.sort_unstable();
                        e.dedup();
                        e
                    })
                })
                .collect(),
            macro_made: Default::default(),
            statics: units.iter().map(|(f, _, s)| (f.to_string(), s.iter().map(|s| s.to_string()).collect())).collect(),
            all: true,
        };
        let (own, _) = split(&ix, &defs);
        for (f, e, st) in &units {
            assert_eq!(w.own(f, e, st), own.get(*f).cloned().unwrap_or_default(), "unit {f}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A unit's key moves only with the records of the fns it defines (the runtime unit's
    /// macro-made ones included), never with a record no unit defines: the runtime unit
    /// keeps its key when another unit's inline copy of a `lust_*` fn comes or goes.
    #[test]
    fn a_unit_key_moves_only_with_its_own_records() {
        let with_copy: [(&str, &[&str]); 3] = [("u1.c", &["f", "lust_copy"]), ("u2.c", &["g"]), ("u3.c", &["lust_abs_i8"])];
        let without: [(&str, &[&str]); 3] = [("u1.c", &["f"]), ("u2.c", &["g"]), ("u3.c", &["lust_abs_i8"])];
        let mk = |defs: &[(&str, &[&str])], f: u64, g: u64, s: u64, abs: u64| {
            split_mode(&[("f", f), ("g", g), ("u1.c;s", s), ("lust_abs_i8", abs), ("lust_copy", 7), ("f_gone", f + g)], defs, &["lust_abs_i8"])
        };
        let base = mk(&with_copy, 1, 2, 3, 4);
        assert_eq!(base.key_part("u2.c", false), mk(&with_copy, 9, 2, 9, 9).key_part("u2.c", false), "others' records");
        assert_ne!(base.key_part("u2.c", false), mk(&with_copy, 1, 5, 3, 4).key_part("u2.c", false), "own record");
        assert_ne!(base.key_part("u1.c", false), mk(&with_copy, 1, 2, 5, 4).key_part("u1.c", false), "own static record");
        assert_ne!(base.key_part("u3.c", false), mk(&with_copy, 1, 2, 3, 5).key_part("u3.c", false), "own macro-made record");
        assert_eq!(base.key_part("u1.c", false), mk(&with_copy, 1, 2, 3, 5).key_part("u1.c", false));
        assert_eq!(base.key_part("u3.c", false), mk(&without, 1, 2, 3, 4).key_part("u3.c", false), "an inline copy elsewhere comes or goes");
        // `f_gone` and, with no copy left, `lust_copy` (no unit defines them) go to no unit.
        let Mode::Use(p) = &mk(&without, 1, 2, 3, 4) else { unreachable!() };
        assert_eq!(p.counts, Some((1, 2)));
    }

    /// A unit with `#include`s is read through the preprocessor: the fns its macros make
    /// are its own.
    #[test]
    fn macro_made_fns_belong_to_their_unit() {
        let Ok(spec) = super::super::spec_for(crate::protocol::host_triple()) else { return };
        let c = "#include <stddef.h>\n#define MK(n) int lust_##n(void) { return 0; }\nMK(a)\nint visible(void) {\n  return 1;\n}\n";
        let pre = super::super::preprocessed(c, &spec.cmd, &super::super::cflags(false), false);
        let d = unit_defs(&[("u2.c".to_string(), &definitions_of(c), pre.as_deref())], true);
        let mut names = d.units[0].1.clone();
        names.sort();
        assert_eq!(names, vec!["lust_a", "visible"]);
        assert_eq!(d.macro_made, ["lust_a".to_string()].into_iter().collect());
    }

    #[test]
    fn llvm_majors_from_version_lines() {
        let zig = "clang version 20.1.2 (https://github.com/ziglang/zig-bootstrap 7ef74e6)";
        assert_eq!(major_after(zig, "clang version "), Ok(20));
        assert_eq!(major_after("Ubuntu clang version 20.1.8 (++20250708)", "clang version "), Ok(20));
        assert_eq!(major_after("Ubuntu LLVM version 20.1.8\n  Optimized build.", "LLVM version "), Ok(20));
        assert_eq!(major_after("LLVM (http://llvm.org/):\n  LLVM version 18.1.3\n", "LLVM version "), Ok(18));
        assert!(major_after("gcc (Ubuntu 13.2.0) 13.2.0", "clang version ").is_err());
    }

    #[test]
    fn quantize_keeps_small_counts_and_cuts_low_bits() {
        let r = "f\n# Func Hash:\n12345\n# Num Counters:\n2\n# Counter Values:\n5\n1000\n# Num Value Kinds:\n1\n# ValueKind = IPVK_IndirectCallTarget:\n0\n# NumValueSites:\n1\n1\ng:1000";
        let q = quantize(r, 3);
        assert!(q.contains("# Func Hash:\n12345\n"), "the hash stays");
        assert!(q.contains("\n5\n896\n"), "{q}");
        assert!(q.ends_with("g:896"), "{q}");
    }

    #[test]
    fn pins_read_llvm_20_summaries() {
        let show = "Detailed summary:\n21764 blocks (0.63%) with count >= 32768 account for 99% of the total counts.\n129141 blocks (3.71%) with count >= 4 account for 99.9999% of the total counts.\n";
        let p = pins(show).unwrap();
        assert!(p.contains(&"-profile-summary-hot-count=32768".to_string()), "{p:?}");
        assert!(p.contains(&"-profile-summary-cold-count=4".to_string()), "{p:?}");
        assert!(p.contains(&"-profile-summary-huge-working-set-size-threshold=0".to_string()), "{p:?}");
    }

    #[test]
    fn pins_read_the_hot_and_cold_cutoffs() {
        let show = "Detailed summary:\n20000 blocks with count >= 40960 account for 99 percentage of the total counts.\n90000 blocks with count >= 10 account for 99.9999 percentage of the total counts.\n";
        let p = pins(show).unwrap();
        assert!(p.contains(&"-profile-summary-hot-count=32768".to_string()), "{p:?}");
        assert!(p.contains(&"-profile-summary-cold-count=8".to_string()), "{p:?}");
        assert!(p.contains(&"-profile-summary-huge-working-set-size-threshold=0".to_string()), "{p:?}");
    }
}
