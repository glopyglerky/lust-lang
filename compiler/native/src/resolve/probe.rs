// Cheap `cargo check` runs for the feature probe and build script records
// (docs/ARCH.md section 1.1).
//
// Lust asks Cargo two things it cannot compute itself: the exact feature
// set of every unit, and the outputs of build scripts. A plain `cargo
// check` answers both but compiles the whole package with rustc (ts_goport:
// about 16 CPU minutes). The probe here runs the same `cargo check` (same
// flags, same target dir) with lust-core as RUSTC_WRAPPER. Cargo still
// resolves the units and features itself, so its JSON messages are the
// same; only rustc changes:
// - host units (build scripts, proc macros, and every crate they depend
//   on) compile for real, because Cargo runs them. They emit `link`;
// - check units (`--emit=dep-info,metadata` into the deps dir) do not run
//   rustc: the wrapper writes an empty dep-info file and deletes the
//   `.rmeta`. Cargo treats a unit whose output is missing as stale, so a
//   later real `cargo check` rebuilds it and never reads a fake output.
// A rustc call a build script makes (autocfg, version probes) writes to its
// OUT_DIR, never to the deps dir, so it always runs for real and
// `rustc-cfg` outputs do not change.
// The probe uses the user's own target dir, so its units have the same
// hashes as the user's builds: build script runs (a C library build, for
// example) and proc macros are shared both ways.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Env var that puts lust-core in wrapper mode; its value is the deps dir
/// whose check units are faked.
const FAKE_DIR_ENV: &str = "LUST_PROBE_FAKE_DIR";

/// A `cargo check --message-format=json --manifest-path <manifest>` command
/// in the manifest's dir, for `triple` (empty: no `--target`, as a plain
/// `cargo check`). Callers add the package, feature, and network flags.
/// `env`: the client's Cargo env (`cargo::apply_cargo_env`), for this
/// command and the build dir lookup.
/// Falls back to a plain check (same messages, full cost) when lust-core
/// cannot be the wrapper: a process that is not lust-core (unit tests), or
/// no build dir from `cargo metadata`.
pub fn check_command(manifest: &Path, triple: &str, env: Option<&[(String, String)]>) -> Command {
    let mut cmd = super::cargo::cargo_command(env);
    if let Some(dir) = manifest.parent() {
        cmd.current_dir(dir);
    }
    cmd.args(["check", "--message-format=json", "--manifest-path"]).arg(manifest);
    if !triple.is_empty() {
        cmd.args(["--target", triple]);
    }
    let exe = std::env::current_exe().ok().filter(|p| p.file_name().is_some_and(|n| n == "lust-core"));
    if let Some(exe) = exe
        && let Some(build) = super::cargo::build_dir(manifest, env)
    {
        // Cargo's layout: `<build dir>/[<triple>/]debug/deps` holds the
        // dev profile's units.
        let profile = if triple.is_empty() { build.join("debug") } else { build.join(triple).join("debug") };
        cmd.env("RUSTC_WRAPPER", exe);
        cmd.env(FAKE_DIR_ENV, profile.join("deps"));
    }
    cmd
}

/// Wrapper mode of lust-core: Cargo runs `lust-core <rustc> <args...>`.
/// Returns None when this process is not a probe wrapper, else the exit code.
pub fn wrapper_main() -> Option<i32> {
    let fake_dir = std::env::var_os(FAKE_DIR_ENV)?;
    let mut argv = std::env::args_os().skip(1);
    let rustc = argv.next()?;
    let args: Vec<OsString> = argv.collect();
    if let Some((dep_info, rmeta)) = fake_outputs(&args, Path::new(&fake_dir)) {
        if let Err(e) = std::fs::write(&dep_info, b"") {
            eprintln!("error: lust probe cannot write {}: {e}", dep_info.display());
            return Some(1);
        }
        // A `.rmeta` from an older real build of this unit must go too:
        // with it, Cargo would take the unit as fresh next time.
        if let Err(e) = std::fs::remove_file(&rmeta)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("error: lust probe cannot remove {}: {e}", rmeta.display());
            return Some(1);
        }
        return Some(0);
    }
    let mut cmd = Command::new(&rustc);
    cmd.args(&args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        eprintln!("error: lust probe cannot run {}: {err}", rustc.to_string_lossy());
        Some(1)
    }
    #[cfg(not(unix))]
    Some(cmd.status().ok().and_then(|s| s.code()).unwrap_or(1))
}

/// The dep-info and `.rmeta` paths of one rustc call, when the call is a
/// check unit of the probe: `--out-dir` is `fake_dir` and `--emit` is
/// exactly dep-info and metadata. None means: run rustc.
fn fake_outputs(args: &[OsString], fake_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let (mut out_dir, mut name, mut extra, mut emit) = (None, None, String::new(), None);
    let mut it = args.iter().map(|a| a.to_str());
    while let Some(a) = it.next() {
        let a = a?;
        match a {
            "--out-dir" => out_dir = Some(it.next()??),
            "--crate-name" => name = Some(it.next()??),
            "--emit" => emit = Some(it.next()??),
            "-C" => {
                if let Some(v) = it.next()?.and_then(|v| v.strip_prefix("extra-filename=")) {
                    extra = v.to_string();
                }
            }
            _ => {
                if let Some(v) = a.strip_prefix("--emit=") {
                    emit = Some(v);
                } else if let Some(v) = a.strip_prefix("-Cextra-filename=") {
                    extra = v.to_string();
                }
            }
        }
    }
    let out_dir = Path::new(out_dir?);
    let mut kinds: Vec<&str> = emit?.split(',').collect();
    kinds.sort_unstable();
    if out_dir != fake_dir || kinds != ["dep-info", "metadata"] {
        return None;
    }
    let name = name?;
    Some((out_dir.join(format!("{name}{extra}.d")), out_dir.join(format!("lib{name}{extra}.rmeta"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn fakes_only_check_units_in_the_deps_dir() {
        let dir = Path::new("/p/debug/deps");
        let unit = [
            "--crate-name",
            "serde",
            "--edition=2021",
            "src/lib.rs",
            "--emit=dep-info,metadata",
            "-C",
            "extra-filename=-0abc",
            "--out-dir",
            "/p/debug/deps",
        ];
        assert_eq!(fake_outputs(&os(&unit), dir), Some((dir.join("serde-0abc.d"), dir.join("libserde-0abc.rmeta"))));
        // A host unit (build script, proc macro, or one of their deps) emits
        // `link` and runs for real.
        let mut link = unit;
        link[4] = "--emit=dep-info,metadata,link";
        assert_eq!(fake_outputs(&os(&link), dir), None);
        link[4] = "--emit=dep-info,link";
        assert_eq!(fake_outputs(&os(&link), dir), None);
        // A build script's own rustc probe (autocfg) writes to its OUT_DIR.
        let mut probe = unit;
        probe[4] = "--emit=metadata";
        probe[8] = "/p/debug/build/x-1/out";
        assert_eq!(fake_outputs(&os(&probe), dir), None);
        // `rustc -vV` and target info queries carry no out dir.
        assert_eq!(fake_outputs(&os(&["-vV"]), dir), None);
    }
}
