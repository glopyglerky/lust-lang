#!/usr/bin/env python3
"""Build census of the ts-rust fork: lust build and run vs Cargo on the pruned ("clean") fork.

The deep census (tsrust_deep.py) removes the items lust cannot check until each lib checks. No
fork bin keeps its `main` there, so this tool makes runnable programs of the fork's own unit
tests, and finds the build, codegen and runtime bugs that come after a check pass.

Steps, on a scratch copy of the fork (never a fork change):
  tests    every `#[test]` fn of each lib (slice-items --tests) is marked `#[doc = "lust-test"]`
           (rustc drops `#[test]` fns outside test builds), `cfg(test)` is turned on in the text,
           the test fns and the private modules on their paths get `pub(crate)`, and the lib
           root gets `pub fn __lust_test(i: usize) -> bool` with one match arm per test. A bin
           `lust_tests_<lib>` runs test i when it gets i + 1 arguments. dev-dependencies become
           dependencies. Cargo and lust build the same text.
  prune    the deep census loop (remove the innermost item of each error, users of removed
           items as cascades) until `lust check --lib` passes. An error in a dispatcher arm
           removes that arm only.
  rustc    `cargo build` of each test bin (dev profile, as lust builds). A rustc error is a
           FALSE ACCEPT of lust (lust checked the same text): recorded, its item removed, and
           back to prune.
  build    `lust build` of each test bin. A failure after a check pass is a BUILD bug: the
           arms are bisected to the one test that makes it fail, it is recorded with the
           error text, its arm is removed, and the build runs again.
  run      each test runs as its own process, once per build, in the lib's
           package dir (as cargo test does), with a fresh TMPDIR. Exit code, stdout, stderr and
           the files written under TMPDIR must be equal. When they differ, the Cargo build runs
           again: a test whose Cargo runs differ is "flaky", not a bug.

Usage: tsrust_build.py --fork DIR --lust PATH --work DIR [--only a,b] [--max-rounds N]
                       [--timeout SECS] [-j N] [--max-bugs N] [--reuse] [--clean] [--profile NAME]
--profile NAME (release, goport, ...): lust builds each test bin with the release driver
(`--profile NAME --target <host>`; LUST_RELEASE_C and LUST_RELEASE_BACKEND pass through from
the environment) and the Cargo side is `cargo build --profile NAME`, so both sides have the
same overflow checks and debug assertions (a debug Cargo oracle sees debug_assert panics a
release build does not have). The prune step's rustc check stays a dev build.
--reuse keeps <work>/fork from an earlier run (fast reruns after a lust fix); --clean removes the
big dirs at the end and keeps the pruned sources as fork-src.tgz (tsrust-full.sh --boat --build
adds it).
Targets default to the fork's library crates in dependency order (tsrust_full.census_libs). Output: <work>/build.json,
a table of bugs and diffs, and the last line
  tsrust-build: <b>/<libs> build, <same>/<ran> tests same, <k> build bugs, <d> diffs, <f> false accepts
scripts/tsrust-full.sh --build [--boat] runs it.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "bench/slices"))
import prune  # noqa: E402
import tsrust_deep  # noqa: E402
import tsrust_full  # noqa: E402

MARK = '#[doc = "lust-test"]'
DISPATCH = "__lust_test"


def log(msg):
    print(f"tsrust-build: {msg}", flush=True)


# ---- tests: make the unit tests a runnable bin ------------------------------------------------

def cfg_on(text):
    """Turn `test` on in every cfg predicate (`cfg(..)`, `cfg_attr(..)`, `cfg!(..)`): the bare
    word `test` becomes `all()`, so `not(test)` is false and `any(test, x)` is true."""
    out, i = [], 0
    for m in re.finditer(r"\bcfg(?:_attr)?!?\s*\(", text):
        if m.start() < i:
            continue
        depth, j = 1, m.end()
        while j < len(text) and depth:
            depth += {"(": 1, ")": -1}.get(text[j], 0)
            j += 1
        inner = re.sub(r'"[^"]*"|\btest\b', lambda t: t.group(0) if t.group(0).startswith('"') else "all()", text[m.end():j])
        out += [text[i:m.end()], inner]
        i = j
    out.append(text[i:])
    return "".join(out)


def items_tool(*args):
    r = subprocess.run([str(prune.ITEMS), *map(str, args)], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"tsrust-build: slice-items {' '.join(map(str, args))}: {r.stderr.strip()}")
    return r.stdout.splitlines()


def find_tests(root):
    """[(file, fn line, path, ignore, returns a value, [(mod file, mod line, private, name)])]"""
    out = []
    for line in items_tool("--tests", root):
        f = line.split(" ")
        if f[0] == "test":
            out.append({"file": f[1], "line": int(f[2]), "path": f[3], "ignore": f[4] == "1", "value": f[5] == "1", "mods": []})
        else:
            out[-1]["mods"].append((f[1], int(f[2]), f[3] == "1", f[4]))
    return out


def add_text(edits, file, line, word, name, ins):
    """Insert `ins` before `<word> <name>` on one line (line count unchanged)."""
    edits.setdefault(file, []).append((line, word, name, ins))


def apply_edits(edits):
    for f, es in edits.items():
        lines = Path(f).read_text().split("\n")
        for line, word, name, ins in sorted(set(es)):
            s = lines[line - 1]
            m = re.search(rf"\b{word}\s+{re.escape(name)}\b", s)
            if not m:
                continue
            at = m.start()
            # `const fn`, `unsafe fn`, `extern "C" fn`: the visibility goes before them.
            while q := re.search(r'(?:\b(?:const|async|unsafe)|\bextern\s+"[^"]*"|\bextern)\s+$', s[:at]):
                at = q.start()
            if not re.search(r"\bpub\b(\([^)]*\))?\s*$", s[:at]):
                lines[line - 1] = s[:at] + ins + s[at:]
        Path(f).write_text("\n".join(lines))


def mark_attrs(text):
    """`#[test]` -> the marker; `#[ignore..]` / `#[should_panic..]` -> doc markers (rustc warns on
    them outside a test fn; lust need not know them)."""
    text = re.sub(r"^(\s*)#\[\s*test\s*\]", lambda m: m.group(1) + MARK, text, flags=re.M)
    text = re.sub(r"^(\s*)#\[\s*ignore\s*(=\s*\"[^\"]*\")?\s*\]", lambda m: m.group(1) + '#[doc = "lust-ignore"]', text, flags=re.M)
    return re.sub(r"^(\s*)#\[\s*should_panic\s*(\([^\]]*\))?\s*\]", lambda m: m.group(1) + '#[doc = "lust-should-panic"]', text, flags=re.M)


def manifests_with_dev_deps(copy):
    """Each package manifest: [dev-dependencies] become [dependencies] entries (a test bin is a
    normal bin). An entry already in [dependencies] stays as it is."""
    for p in copy.rglob("Cargo.toml"):
        if "target" in p.parts:
            continue
        t = p.read_text()
        m = re.search(r"^\[dev-dependencies\]\n((?:(?!\[).*\n?)*)", t, re.M)
        if not m:
            continue
        deps = m.group(1)
        t = t[:m.start()] + t[m.end():]
        have = set(re.findall(r"^([\w-]+)\s*=", (re.search(r"^\[dependencies\]\n((?:(?!\[).*\n?)*)", t, re.M) or [None, ""])[1], re.M))
        keep = "".join(l + "\n" for l in deps.splitlines() if (mm := re.match(r"([\w-]+)\s*=", l)) and mm.group(1) not in have)
        if re.search(r"^\[dependencies\]\n", t, re.M):
            t = re.sub(r"^\[dependencies\]\n", "[dependencies]\n" + keep, t, count=1, flags=re.M)
        else:
            t += "\n[dependencies]\n" + keep
        p.write_text(t)


def prepare(copy, libs):
    """Mark tests and turn cfg(test) on in every .rs file, write the dispatchers and the bins.
    Returns {lib: [test dict with "index"]}."""
    found = {lib["name"]: find_tests(lib["src"]) for lib in libs}
    edits = {}
    for tests in found.values():
        for t in tests:
            name = t["path"].rsplit("::", 1)[-1]
            add_text(edits, t["file"], t["line"], "fn", name, "pub(crate) ")
            for f, line, private, mname in t["mods"]:
                if private:
                    add_text(edits, f, line, "mod", mname, "pub(crate) ")
    apply_edits(edits)
    for p in copy.rglob("*.rs"):
        if "target" not in p.parts:
            t = p.read_text()
            n = mark_attrs(cfg_on(t))
            if n != t:
                p.write_text(n)
    manifests_with_dev_deps(copy)
    for lib in libs:
        tests = found[lib["name"]]
        for i, t in enumerate(tests):
            t["index"] = i
        root = Path(lib["src"])
        arms = []
        for t in tests:
            call = f"crate::{t['path']}()"
            if t["value"]:
                arms.append(f'        {t["index"]} => if let Err(e) = {call} {{ eprintln!("Error: {{e:?}}"); std::process::exit(1) }},')
            else:
                arms.append(f"        {t['index']} => {call},")
        root.write_text(root.read_text() + "\n\n/// lust test dispatcher (scripts/tsrust_build.py): runs test `i`, false when there is none.\n"
                        f"#[doc(hidden)]\npub fn {DISPATCH}(i: usize) -> bool {{\n    match i {{\n" + "\n".join(arms)
                        + "\n        _ => return false,\n    }\n    true\n}\n")
        pkg = Path(lib["manifest"]).parent
        crate = lib["crate"]
        write_driver(lib)
        m = Path(lib["manifest"])
        mt = m.read_text()
        bin_entry = f'\n[[bin]]\nname = "lust_tests_{crate}"\npath = "lust_tests_{crate}.rs"\ntest = false\nbench = false\n'
        # Before the first table after [package]/[lib], so it is a top-level array entry.
        m.write_text(mt + bin_entry)
    return found


def write_driver(lib):
    """The test bin: runs test i when it gets i + 1 arguments; exit 3 when there is none."""
    crate = lib["crate"]
    Path(lib["manifest"]).parent.joinpath(f"lust_tests_{crate}.rs").write_text(
        "fn main() {\n    // Test i runs with i + 1 arguments (lust's env::Args has `count` only, as `lust test`).\n"
        "    let i = std::env::args().count() - 2;\n"
        f"    if !{crate}::{DISPATCH}(i) {{\n        std::process::exit(3);\n    }}\n}}\n")


def dispatcher_span(root):
    lines = Path(root).read_text().split("\n")
    start = next(i for i, l in enumerate(lines, 1) if l.startswith(f"pub fn {DISPATCH}("))
    end = next(i for i in range(start, len(lines) + 1) if lines[i - 1] == "}")
    return start, end


def drop_arms(root, lines_to_drop):
    """Blank dispatcher arms (line numbers stay). Returns the test indexes dropped."""
    lines = Path(root).read_text().split("\n")
    out = []
    for ln in sorted(set(lines_to_drop)):
        m = re.match(r"\s+(\d+) => ", lines[ln - 1])
        if m:
            out.append(int(m.group(1)))
            lines[ln - 1] = ""
    Path(root).write_text("\n".join(lines))
    return out


def arm_lines(root):
    """{test index: line} of the arms still in the dispatcher."""
    s, e = dispatcher_span(root)
    lines = Path(root).read_text().split("\n")
    return {int(m.group(1)): i for i in range(s, e + 1) if (m := re.match(r"\s+(\d+) => ", lines[i - 1]))}


# ---- prune: the deep census loop, dispatcher-aware ------------------------------------------

def prune_lib(lust, lib, copy, work, env, removed, cause_of_name, max_rounds, tool="lust", errs_fn=None):
    """Remove items until `errs_fn` (default: lust check --lib) reports no error. Returns a
    status string ("ok after N rounds" or why it stopped)."""
    root = str(Path(lib["src"]).resolve())
    for rnd in range(1, max_rounds + 1):
        if errs_fn:
            rc, errs, stderr = errs_fn()
        else:
            rc, errs, stderr = tsrust_deep.errors(lust, lib, work, env)
        if rc == 0:
            return f"ok after {rnd - 1} rounds"
        if not errs:
            return f"stopped: exit {rc} with no diagnostic: {tsrust_full.crash_reason({'stderr': stderr, 'rc': rc})}"
        ds, de = dispatcher_span(root)
        in_disp = [e for e in errs if e[1] and str(Path(e[0]).resolve()) == root and ds <= e[1] <= de]
        if in_disp:
            gone = drop_arms(root, [e[1] for e in in_disp])
            for e in in_disp:
                removed.append({"file": os.path.relpath(e[0], copy), "kind": "arm", "name": "-", "lines": 0, "by": tool,
                                "code": e[2], "message": e[3][:160], "cause": [e[2], e[3][:160]], "target": lib["name"]})
            log(f"{lib['name']} round {rnd}: {tool}: dropped {len(gone)} dispatcher arms")
            if len(in_disp) == len(errs):
                continue
        cache = {}
        rest = [e for e in errs if e not in in_disp]
        located = [e for e in rest if e[1] > 0 and Path(e[0]).resolve().is_relative_to(copy) and tsrust_deep.in_item(e, cache)]
        if not located:
            gone = {n for e in rest for n in tsrust_deep.names_in(e[3]) if n in cause_of_name}
            if not gone or not remove_users_keep_dispatcher(copy, root, gone, cause_of_name, removed, lib["name"]):
                if in_disp:
                    continue
                return f"stopped: no located error: {rest[0][2]} {rest[0][3][:160]}"
            log(f"{lib['name']} round {rnd}: {tool}: removed users of {sorted(gone)}")
            continue
        before = len(removed)
        try:
            prune.prune_round(located, tool, removed, copy, [], lambda c, m: None)
        except SystemExit as e:
            return f"stopped: {e}"
        if len(removed) == before:
            return f"stopped: nothing removed for {located[0][2]} {located[0][3][:120]}"
        for r in removed[before:]:
            key = (r["code"], tsrust_full.norm_message(r["message"], copy).removeprefix("unlocated: "))
            parent = next((cause_of_name[n] for n in tsrust_deep.names_in(r["message"]) if n in cause_of_name), None)
            r["cause"] = list(parent or key)
            r["target"] = lib["name"]
            cause_of_name.setdefault(r["name"], tuple(r["cause"]))
        log(f"{lib['name']} round {rnd}: {tool}: {len(errs)} errors, {len(removed) - before} items removed")
    return f"stopped: {max_rounds} rounds"


def remove_users_keep_dispatcher(copy, root, names, cause_of_name, removed, target):
    """tsrust_deep.remove_users, with the dispatcher (at the end of the root file) taken out
    while it runs: an arm that names a removed item is dropped by the check, not by name."""
    text = Path(root).read_text()
    s, _ = dispatcher_span(root)
    cut = text.split("\n")
    head, tail = "\n".join(cut[:s - 3]), "\n".join(cut[s - 3:])
    Path(root).write_text(head)
    try:
        return tsrust_deep.remove_users(copy, names, cause_of_name, removed, target)
    finally:
        Path(root).write_text(Path(root).read_text() + "\n" + tail)


def dev_features(lib, copy):
    """The package's lust dev feature list (the fork's .lust.toml, docs/dev-features.md): Cargo
    builds the test bin with exactly these, so both compilers see the same cfg."""
    rel = os.path.relpath(Path(lib["manifest"]).parent, copy)
    return prune.dev_features_for({"tsrust_dir": str(copy), "src_dir": rel}, env={})[0]


def cargo_errors(lib, copy, env, profile=None):
    """rustc errors of `cargo build --bin lust_tests_<crate>` (dev, or the Cargo profile
    `profile`): (file, line, code, message)."""
    cmd = ["cargo", "build", "--offline", "--bin", f"lust_tests_{lib['crate']}", "-p", lib["pkg"], "--message-format", "json"]
    cmd += ["--profile", profile] if profile else []
    cmd += ["--no-default-features", "--features", ",".join(dev_features(lib, copy))]
    r = subprocess.run(cmd, cwd=copy, env=env, capture_output=True, text=True, timeout=7200)
    out = []
    for line in r.stdout.splitlines():
        try:
            d = json.loads(line)
        except ValueError:
            continue
        msg = d.get("message") if d.get("reason") == "compiler-message" else None
        if not msg or msg.get("level") != "error":
            continue
        sp = next((s for s in msg.get("spans", []) if s.get("is_primary")), None)
        code = (msg.get("code") or {}).get("code") or "rustc"
        if sp is None:
            out.append(("", 0, code, msg["message"]))
            continue
        f = sp["file_name"]
        f = str((copy / f).resolve()) if not os.path.isabs(f) else f
        # A span inside a macro: the outermost expansion site is in the user file.
        exp = sp.get("expansion")
        while exp and not Path(f).resolve().is_relative_to(copy):
            s2 = exp["span"]
            f = str((copy / s2["file_name"]).resolve())
            sp, exp = s2, s2.get("expansion")
        out.append((f, sp["line_start"], code, msg["message"]))
    return r.returncode, out, r.stderr[-3000:]


# ---- build: lust build, bisect the arms that make it fail -----------------------------------

def bin_target(lib):
    pkg = Path(lib["manifest"]).parent
    return {"name": f"{lib['name']}:lust_tests", "kind": "bin", "bin": f"lust_tests_{lib['crate']}", "src": str(pkg / f"lust_tests_{lib['crate']}.rs"),
            "manifest": lib["manifest"]}


# Extra `lust build` flags of the --profile mode (main sets them): the release driver.
RELEASE_ARGS = []


def host_triple():
    return next(ln.split()[1] for ln in subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout.splitlines() if ln.startswith("host:"))


def cargo_out_dir(profile):
    """The dir under the cargo target dir that holds a profile's bins."""
    return {None: "debug", "dev": "debug", "test": "debug", "bench": "release"}.get(profile, profile)


def lust_build(lust, lib, work, env, out_bin, timeout):
    """(ok, short error text, located errors). Located errors are (file, line, code, message)
    in the fork copy, outside the generated driver. A timeout is a hang, never a pass."""
    t = bin_target(lib)
    cmd = tsrust_full.lust_cmd(lust, "build", t, work) + RELEASE_ARGS
    cmd[cmd.index("-o") + 1] = str(out_bin)
    try:
        r = subprocess.run(cmd, cwd=work, env=env, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return False, "HANG: lust build timed out", []
    if r.returncode == 0:
        return True, "", []
    copy = work / "fork"
    diags, located = [], []
    for line in r.stdout.splitlines():
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("kind") == "diagnostic" and d.get("severity") == "error":
            sp = d.get("span") or {}
            f = d.get("file") or ""
            diags.append(f"{d.get('code')} {d.get('message')} @ {tsrust_full.rel(f or '?', copy)}:{sp.get('line', 0)}")
            if f and sp.get("line") and prune.located(f, sp) and Path(f).resolve().is_relative_to(copy) and not Path(f).name.startswith("lust_tests_"):
                located.append((f, sp["line"], d.get("code", "?"), d.get("message", "")))
    # The whole error line (crash_reason cuts at 200 chars; a layout error names a long type).
    errs = [ln.strip() for ln in r.stderr.splitlines() if "error" in ln.lower() or "panicked" in ln]
    text = "; ".join(diags[:3]) if diags else (errs[-1][:2000] if errs else tsrust_full.crash_reason({"stderr": r.stderr, "rc": r.returncode}))
    return False, f"exit {r.returncode}: {text}", located


def bug_key(text):
    """Group key of a build failure: numbers, temp paths and generated C names masked."""
    t = re.sub(r"/[^\s:]*/", "", text)
    t = re.sub(r"\b[a-z_]+[0-9a-f]{6,}\w*", "SYM", t)
    return re.sub(r"\d+", "N", t)[:200]


def bisect_build(lust, lib, work, env, timeout):
    """Find one test whose arm makes `lust build` fail. Arms are blanked in halves on a saved
    root file, restored after each probe. Returns (index or None, error text)."""
    root = Path(lib["src"])
    orig = root.read_text()
    arms = arm_lines(root)
    out_bin = work / "bisect-bin"

    def fails(keep):
        drop = [ln for i, ln in arms.items() if i not in keep]
        lines = orig.split("\n")
        for ln in drop:
            lines[ln - 1] = ""
        root.write_text("\n".join(lines))
        try:
            ok, text, _ = lust_build(lust, lib, work, env, out_bin, timeout)
        finally:
            root.write_text(orig)
        return (not ok), text

    bad, text = fails(set())
    if bad:
        return None, text
    cand = sorted(arms)
    while len(cand) > 1:
        half, rest = cand[:len(cand) // 2], cand[len(cand) // 2:]
        b, t = fails(set(half))
        if b:
            cand, text = half, t
            continue
        b, t = fails(set(rest))
        if b:
            cand, text = rest, t
            continue
        break  # needs tests from both halves: report the first of the set
    if len(cand) == 1:
        _, text = fails({cand[0]})
    return cand[0], text


# ---- run: each test once per build ----------------------------------------------------------

def tree_digest(d, pid=None):
    """Every file under d (relative path, sha256), sorted; `pid` (a pattern) is masked in the paths."""
    rel = (lambda p: pid.sub(b"(PID)", str(p).encode()).decode()) if pid else str
    return sorted((rel(p.relative_to(d)), hashlib.sha256(p.read_bytes()).hexdigest()) for p in d.rglob("*") if p.is_file())


# Rust 1.99 names the OS thread id in its thread messages (`thread 'main' (4764) panicked`,
# `thread 'main' (4764) has overflowed its stack`).
TID = re.compile(rb"^(thread '[^']*') \(\d+\)", re.M)


def run_test(exe, i, cwd, env, tmp, timeout):
    if tmp.exists():
        shutil.rmtree(tmp)
    tmp.mkdir(parents=True)
    e = dict(env, TMPDIR=str(tmp))
    p = subprocess.Popen([str(exe)] + ["x"] * (i + 1), cwd=cwd, env=e, stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.DEVNULL)
    try:
        out, err = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        p.kill()
        p.communicate()
        return ("hang", b"", b""), tree_digest(tmp)
    # Tests that name std::process::id() (temp dir names) print the run's own pid and write
    # files under dirs named with it: mask that exact number, as TID masks thread ids.
    pid = re.compile(rb"(?<![0-9])" + str(p.pid).encode() + rb"(?![0-9])")
    res = (p.returncode, pid.sub(b"(PID)", out), pid.sub(b"(PID)", TID.sub(rb"\1 (TID)", err)))
    return res, tree_digest(tmp, pid)


def compare(a, b):
    (ra, ta), (rb, tb) = a, b
    if ra == rb and ta == tb:
        return "same"
    if ra[0] != rb[0]:
        return f"exit cargo {ra[0]} lust {rb[0]}" + (f"; lust stderr {rb[2][-200:].decode(errors='replace')!r}" if rb[2] != ra[2] else "")
    for k, name in ((1, "stdout"), (2, "stderr")):
        if ra[k] != rb[k]:
            return f"{name} " + tsrust_full_diff(ra[k], rb[k])
    da, db = dict(ta), dict(tb)
    return "files differ: " + ", ".join(sorted(k for k in set(da) | set(db) if da.get(k) != db.get(k))[:5])


def tsrust_full_diff(a, b):
    la, lb = a.decode(errors="replace").splitlines(), b.decode(errors="replace").splitlines()
    for i, (x, y) in enumerate(zip(la, lb)):
        if x != y:
            return f"line {i + 1}: cargo {x[:160]!r} | lust {y[:160]!r}"
    return f"cargo {len(la)} lines, lust {len(lb)} lines"


def run_lib(lib, tests, cargo_exe, lust_exe, work, timeout, jobs):
    cwd = Path(lib["manifest"]).parent
    env = dict(os.environ, CARGO_MANIFEST_DIR=str(cwd), CARGO_PKG_NAME=lib["pkg"], RUST_TEST_THREADS="1")
    env.pop("RUST_BACKTRACE", None)
    live = arm_lines(lib["src"])
    todo = [t for t in tests if t["index"] in live and not t["ignore"]]

    def one(t):
        tmp = work / "run-tmp" / f"{lib['crate']}-{t['index']}"
        a = run_test(cargo_exe, t["index"], cwd, env, tmp, timeout)
        b = run_test(lust_exe, t["index"], cwd, env, tmp, timeout)
        res = compare(a, b)
        if res != "same":
            a2 = run_test(cargo_exe, t["index"], cwd, env, tmp, timeout)
            if compare(a, a2) != "same":
                res = "flaky: " + compare(a, a2)
            elif b[0][0] == "hang":
                res = "HANG: " + res
        shutil.rmtree(tmp, ignore_errors=True)
        return {"test": t["path"], "index": t["index"], "result": res, "cargo_exit": a[0][0]}

    with ThreadPoolExecutor(jobs) as pool:
        return list(pool.map(one, todo))


# ---- main -----------------------------------------------------------------------------------

def libs_of(copy, names):
    meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"], cwd=copy,
                                     capture_output=True, text=True, check=True).stdout)
    out = {}
    for p in meta["packages"]:
        for t in p["targets"]:
            if "lib" in t["kind"]:
                out[p["name"]] = {"name": p["name"], "pkg": p["name"], "crate": t["name"].replace("-", "_"), "kind": "lib", "bin": t["name"],
                                  "src": t["src_path"], "manifest": p["manifest_path"]}
    return [out[n] for n in names]


def check_and_rustc(a, lib, copy, work, env, removed, cause_of_name, report, st):
    """Prune until lust checks the lib and rustc builds the test bin (same text). Removals rustc
    asks for are lust FALSE ACCEPTS. True when both accept."""
    for _ in range(20):
        st["check"] = prune_lib(a.lust, lib, copy, work, env, removed, cause_of_name, a.max_rounds)
        log(f"{lib['name']}: lust check: {st['check']}")
        if not st["check"].startswith("ok"):
            return False
        n0 = len(removed)
        st["rustc"] = prune_lib(a.lust, lib, copy, work, env, removed, cause_of_name, 10, tool="rustc", errs_fn=lambda: cargo_errors(lib, copy, env))
        fa = removed[n0:]
        report["false_accepts"] += fa
        for r in fa:
            log(f"FALSE ACCEPT {lib['name']}: {r['code']} {r['message'][:140]} ({r['file']} `{r['name']}`)")
        if not st["rustc"].startswith("ok"):
            return False
        if not fa:
            return True
    return False


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fork", required=True, type=Path)
    ap.add_argument("--lust", required=True)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--only", default="")
    # The v0.7.0 fork's ts_goport needs about 280 prune rounds.
    ap.add_argument("--max-rounds", type=int, default=600)
    ap.add_argument("--timeout", type=int, default=60, help="seconds per test run")
    ap.add_argument("--build-timeout", type=int, default=1800)
    ap.add_argument("--max-bugs", type=int, default=12, help="build bugs to bisect per lib before giving up")
    ap.add_argument("--clean", action="store_true", help="at the end, remove the fork copy, target dirs, caches and bins (keep build.json)")
    ap.add_argument("--reuse", action="store_true", help="keep <work>/fork from an earlier run (its pruning and tests.json)")
    ap.add_argument("-j", type=int, default=1, help="test runs at once (the fork's tests share files: 1 is safe)")
    ap.add_argument("--profile", help="build both sides with this Cargo profile (lust: the release driver)")
    a = ap.parse_args()
    if a.profile:
        RELEASE_ARGS[:] = ["--profile", a.profile, "--target", host_triple()]
    prune.build_items_tool()
    work = a.work.resolve()
    copy = work / "fork"
    env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(work / "lust-cache"), CARGO_TARGET_DIR=str(work / "cargo-target"))
    a.only = a.only or tsrust_full.census_libs(a.fork.resolve())
    if a.reuse and (work / "tests.json").exists():
        libs = libs_of(copy, a.only.split(","))
        tests = json.loads((work / "tests.json").read_text())
        for lib in libs:
            write_driver(lib)
    else:
        if copy.exists():
            shutil.rmtree(copy)
        shutil.copytree(a.fork.resolve(), copy, ignore=shutil.ignore_patterns("target", ".git", ".lust-cache"))
        libs = libs_of(copy, a.only.split(","))
        tests = prepare(copy, libs)
        (work / "tests.json").write_text(json.dumps(tests))
    for lib in libs:
        log(f"{lib['name']}: {len(tests[lib['name']])} tests")
    removed, cause_of_name = [], {}
    report = {"libs": {}, "removed": removed, "false_accepts": [], "build_bugs": [], "runs": []}

    def save():
        (work / "build.json").write_text(json.dumps(report, indent=0))

    (work / "bin").mkdir(exist_ok=True)
    for lib in libs:
        st = report["libs"].setdefault(lib["name"], {"tests": len(tests[lib["name"]])})
        lust_exe = work / "bin" / f"lust_tests_{lib['crate']}"
        ok, bugs = False, 0
        while bugs <= a.max_bugs:
            if not check_and_rustc(a, lib, copy, work, env, removed, cause_of_name, report, st):
                break
            ok, text, located = lust_build(a.lust, lib, work, env, lust_exe, a.build_timeout)
            if ok:
                break
            bugs += 1
            if located:
                # A build-only error at a span (a mono-time gap): remove its item, as the census
                # does for check errors, and check again (cascades).
                n0 = len(removed)
                try:
                    prune.prune_round(located, "lust-build", removed, copy, [], lambda c, m: None)
                except SystemExit as e:
                    log(f"{lib['name']}: cannot remove the build error's item: {e}")
                    break
                for r in removed[n0:]:
                    r["cause"], r["target"] = [r["code"], tsrust_full.norm_message(r["message"], copy)], lib["name"]
                    cause_of_name.setdefault(r["name"], tuple(r["cause"]))
                    report["build_bugs"].append({"lib": lib["name"], "test": None, "item": f"{r['file']} `{r['name']}`", "error": text, "key": bug_key(text)})
                log(f"BUILD BUG {lib['name']}: {text[:300]} (removed {len(removed) - n0} items)")
                save()
                continue
            idx, text = bisect_build(a.lust, lib, work, env, a.build_timeout)
            name = next((t["path"] for t in tests[lib["name"]] if t["index"] == idx), None)
            report["build_bugs"].append({"lib": lib["name"], "test": name, "error": text, "key": bug_key(text)})
            log(f"BUILD BUG {lib['name']}: {name or '(fails with no test)'}: {text[:300]}")
            save()
            if idx is None:
                break
            drop_arms(lib["src"], [arm_lines(lib["src"])[idx]])
        st["arms"] = len(arm_lines(lib["src"]))
        st["build"] = "ok" if ok else "fail"
        if not ok:
            save()
            continue
        cargo_exe = work / "cargo-target" / cargo_out_dir(a.profile) / f"lust_tests_{lib['crate']}"
        rc, errs, stderr = cargo_errors(lib, copy, env, a.profile)  # the arms may have changed
        if rc != 0:
            st["build"] = f"cargo build failed after the lust build: {errs[:1]} {stderr[-300:]}"
            save()
            continue
        runs = run_lib(lib, tests[lib["name"]], cargo_exe, lust_exe, work, a.timeout, a.j)
        for r in runs:
            r["lib"] = lib["name"]
        report["runs"] += runs
        st["ran"] = len(runs)
        st["same"] = sum(r["result"] == "same" for r in runs)
        log(f"{lib['name']}: {st['same']}/{st['ran']} tests same")
        for r in runs:
            if r["result"] != "same" and not r["result"].startswith("flaky"):
                log(f"DIFF {lib['name']} {r['test']}: {r['result'][:300]}")
        save()
    save()
    keys = {}
    for b in report["build_bugs"]:
        keys.setdefault(b["key"], []).append(b)
    print("\nbuild bugs by root cause:")
    for k, v in sorted(keys.items(), key=lambda kv: -len(kv[1])):
        print(f"  {len(v)}x {k}\n     first: {v[0]['lib']} {v[0]['test']}")
    diffs = [r for r in report["runs"] if r["result"] != "same" and not r["result"].startswith("flaky")]
    nb = sum(st.get("build") == "ok" for st in report["libs"].values())
    same = sum(r["result"] == "same" for r in report["runs"])
    log(f"{nb}/{len(libs)} build, {same}/{len(report['runs'])} tests same, {len(report['build_bugs'])} build bugs, "
        f"{len(diffs)} diffs, {len(report['false_accepts'])} false accepts")
    if a.clean:
        # The pruned sources stay, for repros (a few MB): fork-src.tgz.
        subprocess.run(["tar", "-czf", str(work / "fork-src.tgz"), "--exclude=target", "--exclude=.lust-cache", "-C", str(work), "fork"])
        for d in ("fork", "cargo-target", "lust-cache", "run-tmp", "bin", "bisect-bin"):
            p = work / d
            shutil.rmtree(p, ignore_errors=True) if p.is_dir() else p.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
