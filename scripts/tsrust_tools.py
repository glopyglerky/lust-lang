#!/usr/bin/env python3
"""The ts-rust fork targets that check and build under lust, kept there by the gate.

Gate form (no argument):
  tools  the pinned copy in bench/tsrust-tools (the fork's two tool crates, their inputs, and a
         workspace manifest; no dependencies) is checked by lust (both libs and both bins), the
         two bins are built by lust and by Cargo, and the parity runner's tool runs
         (tsrust_parity.TOOL_RUNS) must give the same exit code, stdout, stderr and files.
  libs   lust checks the fork libs (tsrust_full.census_libs) from the pinned fork tree: third_party/ts-rust when it
         is checked out at the pin, else the tar <pin>.tgz in ~/fork-tar (remote-gate.sh copies
         it to the gate box) or in FORK_TARS. No tree is a failure, never a skip.
  tsgo   lust builds the tsgo bin from the pinned fork, unpacked at REF_ROOT/ts-rust, and its
         parity rows run against the Cargo reference <pin>.tsgo-ref.tzst (next to the fork tar,
         unpacked at REF_ROOT/target): every tsgo fixture run (tsrust_parity.parity_runs) and
         the integration test bins with the lust tsgo in place of Cargo's (parity_tests), cut
         to the fixed subset TSGO_* below (the lust tsgo is a dev build and runs about 70x
         slower than Cargo's; the hourly census runs all of them). The
         paths are fixed because the Cargo test bins keep the paths they were built at
         (env!("CARGO_MANIFEST_DIR"), CARGO_BIN_EXE_<bin>). No reference is a failure.
A lust timeout is a hang: a failure, never a reject.
Last line: "tsrust-tools: <ok> passed, <bad> failed".

--refresh: copy the tools and their inputs again from third_party/ts-rust (the pinned fork).
Run it after a fork bump that changes tools/, then commit bench/tsrust-tools.
--make-ref DIR: Cargo-build the tsgo reference of the pin at REF_ROOT and write
DIR/<pin>.tsgo-ref.tzst. Needs a boat sandbox (REF_ROOT is in its home); run it through
`scripts/tsrust-tools.sh --make-ref`, which stores the tar in FORK_TARS. Once per fork pin.
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import tsrust_full  # noqa: E402
import tsrust_parity  # noqa: E402

ROOT = HERE.parent
PINNED = ROOT / "bench/tsrust-tools"
FORK_TARS = Path("/home/theo/lust-scratch/tsrust-full/fork-tars")
FORK_MIRROR = Path("/home/theo/lust-scratch/tsrust-full/fork-mirror.git")
# Where the tsgo rows unpack the fork and the Cargo reference (the gate box's home; a box with
# several gate parts sets HOME elsewhere, so the tar dirs list /home/user/fork-tar too).
REF_ROOT = Path("/home/user/tsgo-ref")
TAR_DIRS = [Path.home() / "fork-tar", Path("/home/user/fork-tar"), FORK_TARS]
# The reference is 9 bins that share most of their code: a long zstd window packs 464 MB into
# 38 MB (gzip: 127 MB), so it uploads to each gate box in a few seconds.
ZSTD_C, ZSTD_D = "zstd -19 -T0 --long=29", "zstd -d --long=29"
# The gate's tsgo parity subset, fixed by name so one list fits both fork pins (names a pin
# lacks match nothing). Lust tsgo seconds per project (4 runs, boat): v0.7.0 (ae1c76940) heavy
# projects take about 15 s, light ones 1-5 s; the first cut (140 runs + 6 test bins) took 546 s. Kept:
# every fixtures/ project but TSGO_SKIP (the lib and side sub-projects of the build_watch and
# watch-stale-dts fixtures, built anyway by their parent's -b run, and 5 of the 7 emit_context
# targets: es, one of the three that panic in both builds, and legacy stay), the effect
# fixtures in TSGO_EFFECT, and every test bin but early_emit (256 s on v0.7.0; its tsgo emit of
# fixtures/emit_pool is in the runs). v0.7.0: 120 of 216 runs, 6 of 7 test bins, about 180 s;
# 61a13796b: 80 of 140 runs.
TSGO_SKIP = {f"fixtures/{p}" for p in [
    "multiprog/watch-stale-dts/lib", "multiprog/watch-stale-dts/side",
    "build_watch/coarse-mtime/lib", "build_watch/coarse-mtime/side",
    "build_watch/stale-json/lib", "build_watch/stale-json/side",
    "emit_context/es-vms", "emit_context/js-split", "emit_context/legacy-param", "emit_context/deco-legacy", "emit_context/legacy-vms",
]}
TSGO_EFFECT = {f"effect_fixtures/{p}" for p in [
    "effect-error", "ignore-errors-keeps-ts",
    "unstable-api-usage", "api-stability-leak", "experimental-api-usage", "allowed-unstable-apis",
]}
TSGO_TESTS = [t for t in tsrust_parity.TESTS if t != "early_emit"]
TOOLS = ["ts_ast_codegen", "ts_diagnostics_codegen"]
# Fork files the tool runs read (tsrust_parity.TOOL_RUNS), at the same relative paths.
INPUTS = [
    "crates/ts_goport/src/astdata/syntax_kind.rs",
    "crates/ts_goport/src/astdata/ast_generated.rs",
    "crates/ts_goport/data/effect/effectDiagnosticMessages.json",
]


def refresh():
    fork = ROOT / "third_party/ts-rust"
    if not (fork / "Cargo.toml").exists():
        sys.exit("tsrust-tools: third_party/ts-rust is not checked out")
    if PINNED.exists():
        shutil.rmtree(PINNED)
    for t in TOOLS:
        shutil.copytree(fork / "tools" / t, PINNED / "tools" / t)
    for f in INPUTS:
        (PINNED / f).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(fork / f, PINNED / f)
    # The workspace keeps the fork's package and lint tables, with only the two tools.
    ws = tomllib.loads((fork / "Cargo.toml").read_text())["workspace"]
    text = (fork / "Cargo.toml").read_text()
    start = text.index("[workspace.package]")
    members = "".join(f'    "tools/{t}",\n' for t in TOOLS)
    head = f'# Pinned copy of the ts-rust fork\'s codegen tools (scripts/tsrust_tools.py --refresh).\n[workspace]\nresolver = "{ws.get("resolver", "2")}"\nmembers = [\n{members}]\n\n'
    (PINNED / "Cargo.toml").write_text(head + text[start:])
    head_rev = subprocess.run(["git", "-C", str(fork), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    (PINNED / "FORK_REV").write_text(head_rev + "\n")
    r = subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=PINNED)
    print(f"tsrust-tools: refreshed {PINNED.relative_to(ROOT)} from fork {head_rev[:9]}")
    return r.returncode


def fork_pin():
    """The fork commit HEAD pins; TSRUST_FORK_PIN when set (a boat sandbox copy is not a git repo)."""
    if os.environ.get("TSRUST_FORK_PIN"):
        return os.environ["TSRUST_FORK_PIN"]
    r = subprocess.run(["git", "-C", str(ROOT), "ls-tree", "HEAD", "third_party/ts-rust"], capture_output=True, text=True)
    return r.stdout.split()[2] if r.returncode == 0 and r.stdout.split() else None


def fork_tree(tmp):
    """The pinned fork tree: the checked-out submodule at the pin, else an unpacked <pin>.tgz.
    (None, reason) when there is none."""
    pin = fork_pin()
    if pin is None:
        return None, "no third_party/ts-rust pin in HEAD"
    sub = ROOT / "third_party/ts-rust"
    if (sub / "Cargo.toml").exists():
        r = subprocess.run(["git", "-C", str(sub), "rev-parse", "HEAD"], capture_output=True, text=True)
        if r.stdout.strip() == pin:
            return sub, None
    tar, why = fork_tar(pin)
    return (None, why) if tar is None else unpack(tar, tmp / "ts-rust")


def find_tar(name):
    return next((d / name for d in TAR_DIRS if (d / name).exists()), None)


def unpack(tar, dest):
    """Unpack tar into the new dir dest: (dest, None), or (None, reason)."""
    dest.mkdir(parents=True)
    how = ["-I", ZSTD_D] if tar.suffix == ".tzst" else ["-z"]
    if subprocess.run(["tar", *how, "-xf", str(tar), "-C", str(dest)]).returncode != 0:
        return None, f"cannot unpack {tar}"
    return dest, None


def fork_tar(pin):
    """The fork tar <pin>.tgz: (path, None), or (None, reason)."""
    # A local gate (no remote-gate.sh upload): make the cached tar from the fork mirror.
    if not (FORK_TARS / f"{pin}.tgz").exists() and FORK_MIRROR.exists():
        FORK_TARS.mkdir(parents=True, exist_ok=True)
        tmp_tar = FORK_TARS / f".new-{pin}-{os.getpid()}"
        r = subprocess.run(f"set -o pipefail; git -C {FORK_MIRROR} archive --format=tar {pin} | gzip -1 > {tmp_tar}", shell=True, executable="/bin/bash")
        if r.returncode == 0:
            os.replace(tmp_tar, FORK_TARS / f"{pin}.tgz")
        else:
            tmp_tar.unlink(missing_ok=True)
    tar = find_tar(f"{pin}.tgz")
    if tar is None:
        return None, f"no fork tree at pin {pin[:9]} (no checked-out submodule, no {pin[:9]}.tgz in {', '.join(map(str, TAR_DIRS))})"
    return tar, None


def ref_tree(with_ref):
    """A fresh REF_ROOT with the pinned fork at REF_ROOT/ts-rust, and the Cargo reference at
    REF_ROOT/target when with_ref: (fork dir, None), or (None, reason)."""
    pin = fork_pin()
    if pin is None:
        return None, "no third_party/ts-rust pin in HEAD"
    tar, why = fork_tar(pin)
    if tar is None:
        return None, why
    ref = find_tar(f"{pin}.tsgo-ref.tzst")
    if with_ref and ref is None:
        return None, f"no Cargo reference {pin[:9]}.tsgo-ref.tzst in {', '.join(map(str, TAR_DIRS))} (make it: scripts/tsrust-tools.sh --make-ref)"
    try:
        shutil.rmtree(REF_ROOT, ignore_errors=True)
        REF_ROOT.mkdir(parents=True)
    except OSError as e:
        return None, f"cannot make {REF_ROOT} (the tsgo rows run on a boat gate box): {e}"
    fork, why = unpack(tar, REF_ROOT / "ts-rust")
    if fork is not None and with_ref:
        _, why = unpack(ref, REF_ROOT / "target")
    return (None, why) if why else (fork, None)


def make_ref(dest):
    """Cargo-build the tsgo reference at REF_ROOT (the census's cargo steps without the test
    runs, so the bins and test bins are the ones its parity used) and pack the 9 bins and the
    test bins into dest/<pin>.tsgo-ref.tzst."""
    fork, why = ref_tree(with_ref=False)
    if fork is None:
        sys.exit(f"tsrust-tools: {why}")
    env = {**os.environ, "CARGO_TARGET_DIR": str(REF_ROOT / "target")}
    for cmd in (["cargo", "test", "--release", "--workspace", "--offline", "--no-run"],
                ["cargo", "build", "--release", "--workspace", "--bins", "--offline"]):
        if subprocess.run(cmd, cwd=fork, env=env).returncode != 0:
            sys.exit(f"tsrust-tools: {' '.join(cmd)} failed")
    rel = REF_ROOT / "target/release"
    exes = [tsrust_parity.test_exe(rel / "deps", t) for t in tsrust_parity.TESTS]
    if None in exes:
        sys.exit("tsrust-tools: a test bin is missing after cargo test --no-run")
    files = [f"release/{b}" for b in tsrust_parity.BINS] + [str(e.relative_to(REF_ROOT / "target")) for e in exes]
    dest.mkdir(parents=True, exist_ok=True)
    tar = dest / f"{fork_pin()}.tsgo-ref.tzst"
    r = subprocess.run(["tar", "-I", ZSTD_C, "-cf", str(tar), "-C", str(REF_ROOT / "target"), *files])
    print(f"tsrust-tools: wrote {tar} ({tar.stat().st_size >> 20} MB)" if r.returncode == 0 else "tsrust-tools: tar failed")
    return r.returncode


def tsgo_rows(lust, timeout, out):
    """lust build of tsgo, then its parity rows vs the Cargo reference (docstring "tsgo"):
    (ok count, failure lines). Off a boat box (no REF_ROOT's parent: a local gate slot) the
    rows are skipped with a printed reason, not failed: the reference's test bins only run
    from the path they were built at, and boat gates run them."""
    if not REF_ROOT.parent.is_dir():
        print(f"tsrust-tools: skip tsgo: {REF_ROOT.parent} is not here (the tsgo rows run on a boat gate box)", flush=True)
        return 0, []
    fork, why = ref_tree(with_ref=True)
    if fork is None:
        return 0, [f"tsgo: {why}"]
    t = next(t for t in tsrust_full.targets(fork) if t["name"] == "ts_goport:tsgo")
    r = tsrust_full.run_lust(lust, "build", t, out, timeout)
    line = f"build tsgo: {r['status']} ({r.get('secs', '?')}s)"
    print(f"tsrust-tools: {line}", flush=True)
    if r["status"] != "ok":
        return 0, [line + f" {r['diags'][0]['message'][:120] if r['diags'] else r['stderr'][-200:]}"]
    cargo_bins, t0 = REF_ROOT / "target/release", time.time()
    keep = lambda case: case not in TSGO_SKIP and (case.startswith("fixtures/") or case in TSGO_EFFECT)  # noqa: E731
    runs = tsrust_parity.parity_runs(fork, cargo_bins, out / "bin", out, timeout, bins=["tsgo"], keep=keep)
    tests = tsrust_parity.parity_tests(fork, cargo_bins, out / "bin", cargo_bins / "deps", timeout * 4, swap=["tsgo"], tests=TSGO_TESTS)
    same = lambda rows: sum(res == "same" for _, res in rows)  # noqa: E731
    print(f"tsrust-tools: parity tsgo: {same(runs)}/{len(runs)} runs same, {same(tests)}/{len(tests)} test bins same ({time.time() - t0:.0f}s)", flush=True)
    rows = runs + tests
    shutil.rmtree(REF_ROOT, ignore_errors=True)
    return 1 + same(rows), [f"parity tsgo {tag}: {res}" for tag, res in rows if res != "same"]


def check_libs(lust, timeout, out):
    """lust check of each fork lib (tsrust_full.census_libs) from the pinned fork tree:
    (ok count, failure lines)."""
    fork, why = fork_tree(out)
    if fork is None:
        return 0, [f"libs: {why}"]
    by_name = {t["name"]: t for t in tsrust_full.targets(fork)}
    ok, bad = 0, []
    for name in tsrust_full.census_libs(fork).split(","):
        r = tsrust_full.run_lust(lust, "check", by_name[name], out, timeout)
        line = f"check {name}: {r['status']} ({r.get('secs', '?')}s)"
        print(f"tsrust-tools: {line}", flush=True)
        if r["status"] == "ok":
            ok += 1
        else:
            bad.append(line + f" {r['diags'][0]['message'][:120] if r['diags'] else r['stderr'][-200:]}")
    return ok, bad


def gate(lust, timeout):
    bad, ok = [], 0
    with tempfile.TemporaryDirectory(prefix="tsrust-tools.") as tmp:
        out = Path(tmp)
        (out / "bin").mkdir()
        work = out / "fork"
        shutil.copytree(PINNED, work)
        ts = tsrust_full.targets(work)
        for t in ts:
            r = tsrust_full.run_lust(lust, "check", t, out, timeout)
            line = f"check {t['name']}: {r['status']}"
            if r["status"] == "ok":
                ok += 1
            else:
                bad.append(line + f" {r['diags'][0]['message'][:120] if r['diags'] else r['stderr'][-200:]}")
            print(f"tsrust-tools: {line}", flush=True)
        for t in (t for t in ts if t["kind"] == "bin"):
            r = tsrust_full.run_lust(lust, "build", t, out, timeout)
            line = f"build {t['name']}: {r['status']}"
            if r["status"] == "ok":
                ok += 1
            else:
                bad.append(line + f" {r['diags'][0]['message'][:120] if r['diags'] else r['stderr'][-200:]}")
            print(f"tsrust-tools: {line}", flush=True)
        c = subprocess.run(["cargo", "build", "--release", "--offline", "--bins", "-q"], cwd=work,
                           env={**os.environ, "CARGO_TARGET_DIR": str(out / "cargo-target")}, capture_output=True, text=True)
        if c.returncode != 0:
            bad.append(f"cargo build failed: {c.stderr[-300:]}")
        else:
            for tag, res in tsrust_parity.parity_tools(work, out / "cargo-target/release", out / "bin", out, timeout):
                if res == "same":
                    ok += 1
                else:
                    bad.append(f"parity {tag}: {res}")
        for part in (check_libs, tsgo_rows):
            part_ok, part_bad = part(lust, timeout, out)
            ok += part_ok
            bad += part_bad
    for b in bad:
        print(f"tsrust-tools: FAIL {b}")
    print(f"tsrust-tools: {ok} passed, {len(bad)} failed")
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--refresh", action="store_true")
    ap.add_argument("--make-ref", type=Path, metavar="DIR")
    ap.add_argument("--lust", default=str(ROOT / "target/release/lust"))
    ap.add_argument("--timeout", type=int, default=300)
    a = ap.parse_args()
    sys.exit(refresh() if a.refresh else make_ref(a.make_ref) if a.make_ref else gate(a.lust, a.timeout))


if __name__ == "__main__":
    main()
