#!/usr/bin/env python3
"""Warm one-fn edit bench and stale-cache probe on real ts-rust code: lust daemon vs cargo check.

The north star is a warm one-fn edit check 100x faster than `cargo check`, and rule zero: lust
never accepts a program rustc rejects. scripts/speed_slice.py measures body edits on slices;
this measures several edit KINDS on the largest real code that checks today, and checks that
the daemon's cached answer is never stale.

Trees (one per run):
  --fork DIR --target NAME   a ts-rust fork tree (usually the census clean fork:
                             scripts/tsrust_deep.py --rustc --save-clean; --clean TGZ unpacks
                             that file over a copy of DIR) and one lib or bin target of it
  --slice bench/slices/NAME  a pinned slice, as a one-crate tree (speed_slice.py's crate)

Edit kinds (each probe edits one fn or item, then goes back to the base text):
  body     a `let` in one fn body; reject: a `let` of the wrong type
  infer    a `let` whose element type is inferred from a later use, then a use that changes
           that type; reject: a use no impl fits (`u64: Sum<&u8>`)
  sig      a real fn's signature: `where u32: Copy`; rejects: `where String: Copy`, one
           more parameter (callers break)
  sigx     a crate-root fn called from fns in other modules: its return and param types
           change (callers still fit through `.into()`); reject: a type callers cannot take
  oblig    a body asks a trait of a real type (`_lust_need_debug::<T>()`): one it derives,
           then one it does not
  derive   one trait removed from a real `#[derive(..)]` (users of that trait break)
  generic  a body edit in a generic fn named in other files; reject: `Option<T>::cloned`
  rpit     a fn that returns `impl Trait`: body edit, `+ Sized` bound; reject: `+ Copy`
  macbody  `body` in fns whose bodies call a pure builtin macro (`matches!`, `write!`,
           `writeln!`, `debug_assert*!`): the expander runs on the patch path (not default)
  blockbody  `body` in fns whose bodies hold a type item (struct, enum, union, trait, type
           alias, impl; site flag `b`): the edit does not touch the item (not default)

Manifest kinds (stale Cargo view probes; not in the default list, `--kinds manifest` runs them all).
Each state edits what Cargo reads besides the sources, and often a cfg-gated bad `let` in one body:
  mfeat     a new feature `_lust_f` gates a bad `let`; on (package `default`, or the .lust.toml
            `dev-features` list when it has one), then off
  medition  the package edition becomes 2021 (lust rejects: L0530)
  mlints    `non_ascii_idents = "deny"` (package `[lints.rust]`, or `[workspace.lints.rust]` when
            the package inherits its lints; lust rejects: L0531), then `warn`
  mprofile  `#[cfg(debug_assertions)]` gates a bad `let`; the workspace root's `[profile.dev]`
            gets `debug-assertions = false`, then loses it
  mdeps     a registry dependency's version requirement: `=X.Y.Z` to `X.Y` (same lock), then a
            version no registry has (both reject)
  mtarget   target discovery: a new build.rs sets cfg `_lust_bs`, a body gates a bad `let` on it,
            then build.rs goes (lust rejects the unknown cfg: L0520). A package that has a build.rs
            gets a new `src/bin` and `examples` target instead.

Setup (not measured): the bench adds helper fns (oblig: one `_lust_need_*` fn per trait at
the end of a file; sigx: `_lust_sigx` at the crate root and calls in other modules), finds the
files rustc compiles (one cargo check with a `compile_error!` in every candidate file), and
warms the daemon and the cargo target dir.

Every state: the warm lust daemon check, `cargo check` of the same target in a warm target
dir (same features as lust: .lust.toml dev features), and (unless --no-cold) a cold one-shot
lust check with an empty cache. Classes per state: lust and rustc verdicts (false accept =
lust ok, rustc error: a rule-zero bug), warm answer vs cold answer (exit, stdout, stderr
without timing lines), and hangs (a timeout is never a reject).

Output: a table per kind (lust median/p95 vs cargo median/p95 over states rustc accepts, and
the ratios), the soundness counts, `--json FILE` with every state, and one summary line:
  tsedit: <tree> <lines> lines: <kind> <ratio med>x/<ratio p95>x ...; <n> states, <fa> false
  accepts, <fr> false rejects, <wc> warm!=cold, <h> hangs
Exit 1 when any false accept, warm!=cold, or hang was seen (a false reject alone is not an
exit failure: lust may be a subset).

Usage: tsedit.py (--fork DIR --target NAME[,NAME] [--clean TGZ|auto] | --slice DIR) [--kinds a,b]
                 [--probes N] [--quick] [--seed N] [--no-cold] [--json FILE] [--out DIR] [--keep]
       tsedit.py --render docs/speed.md a.json [b.json ...]
Full runs on boat: scripts/tsrust-full.sh --boat --tsedit (census, then every lib target).
The gate's speed row (scripts/speed-row.sh) runs one probe per kind on goport-util-small.
"""
import argparse
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import tarfile
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(ROOT / "bench/slices"))
import prune  # noqa: E402  (bench/slices/prune.py: the slice-items tool)
import speed_slice  # noqa: E402
import tsrust_full  # noqa: E402
import oracle_log  # noqa: E402  (scripts/oracle_log.py)

KINDS = ["body", "infer", "sig", "sigx", "oblig", "derive", "generic", "rpit"]
MANIFEST_KINDS = ["mfeat", "medition", "mlints", "mprofile", "mdeps", "mtarget"]
# Kinds outside the default list.
EXTRA_KINDS = ["macbody", "blockbody"]
ALL_KINDS = KINDS + EXTRA_KINDS + MANIFEST_KINDS
# Traits the oblig kind can ask for, with the path its helper names.
NEED = {"Clone": "::std::clone::Clone", "Copy": "::std::marker::Copy", "Debug": "::std::fmt::Debug",
        "Default": "::std::default::Default", "Hash": "::std::hash::Hash", "PartialEq": "::std::cmp::PartialEq",
        "Eq": "::std::cmp::Eq", "PartialOrd": "::std::cmp::PartialOrd", "Ord": "::std::cmp::Ord"}
WARM_TIMEOUT = 900
COLD_TIMEOUT = 1800


def sites_of(path):
    """slice-items --sites: (fns, derives) of one file, as dicts."""
    r = subprocess.run([str(prune.ITEMS), "--sites", str(path)], capture_output=True, text=True)
    if r.returncode != 0:
        return [], []
    fns, derives = [], []
    for line in r.stdout.splitlines():
        f = line.split()
        if f[0] == "fn":
            n = list(map(int, f[1:13]))
            fns.append({"file": path, "ol": n[0], "oc": n[1], "cl": n[2], "cc": n[3], "sig": n[4], "pl": n[5], "pc": n[6],
                        "rl": n[7], "rc": n[8], "wl": n[9], "wc": n[10], "inputs": n[11],
                        "flags": "" if f[13] == "-" else f[13], "name": f[14],
                        "tparams": [] if f[15] == "-" else f[15].split(","), "depth": int(f[16])})
        elif f[0] == "derive":
            derives.append({"file": path, "line": int(f[1]), "depth": int(f[2]), "kind": f[3], "name": f[4],
                            "generic": f[5] == "1", "traits": f[6].split(",")})
    return fns, derives


def insert_at(text, line, idx, snippet):
    """Insert `snippet` at 0-based char index `idx` of 1-based `line`."""
    lines = text.split("\n")
    s = lines[line - 1]
    lines[line - 1] = s[:idx] + snippet + s[idx:]
    return "\n".join(lines)


def in_body(text, fn, snippet):
    """`snippet` right after the fn body's `{` (same line: no other line moves)."""
    return insert_at(text, fn["ol"], fn["oc"], " " + snippet)


def add_param(text, fn):
    """One more parameter `_lust_p: u32` before the `)` that closes the inputs."""
    lines = text.split("\n")
    before = "\n".join(lines[:fn["pl"] - 1] + [lines[fn["pl"] - 1][:fn["pc"] - 1]]).rstrip()
    sep = "" if before.endswith(("(", ",")) else ", "
    return insert_at(text, fn["pl"], fn["pc"] - 1, sep + "_lust_p: u32")


def add_where(text, fn, bound):
    """`bound` added to the fn's where clause (a new clause before the body's `{` if none)."""
    if fn["wl"]:
        return insert_at(text, fn["wl"], fn["wc"] - 1 + len("where"), f" {bound},")
    return insert_at(text, fn["ol"], fn["oc"] - 1, f"where {bound} ")


def toml_add(text, section, line):
    """`line` as the first line of `[section]`; the section is appended when missing."""
    lines = text.split("\n")
    for i, l in enumerate(lines):
        if l.strip() == f"[{section}]":
            return "\n".join(lines[:i + 1] + [line] + lines[i + 1:])
    return text.rstrip("\n") + f"\n\n[{section}]\n{line}\n"


def toml_section(text, section):
    """The lines of `[section]` (empty when missing)."""
    out, on = [], False
    for l in text.split("\n"):
        if l.strip().startswith("["):
            on = l.strip() == f"[{section}]"
        elif on:
            out.append(l)
    return out


def toml_list_add(text, section, key, item):
    """`item` added to the one-line list `key = [..]` of `[section]` ("" = the top level), or a new
    `key = [item]` line when there is none."""
    lines, on = text.split("\n"), section == ""
    for i, l in enumerate(lines):
        if l.strip().startswith("["):
            on = l.strip() == f"[{section}]"
        elif on and (m := re.match(rf"(\s*{re.escape(key)}\s*=\s*\[)\s*(.*)$", l)):
            rest = m.group(2)
            lines[i] = m.group(1) + f'"{item}"' + ("" if rest.startswith("]") else ", ") + rest
            return "\n".join(lines)
    if section == "":
        return f'{key} = ["{item}"]\n' + text
    return toml_add(text, section, f'{key} = ["{item}"]')


def ret_text(text, fn):
    lines = text.split("\n")
    return "\n".join(lines[fn["sig"] - 1:fn["rl"]])


def write_file(path, text):
    """Writes `text` to `path` (None deletes it). Returns the dirs it had to make."""
    if text is None:
        path.unlink(missing_ok=True)
        return []
    made = [d for d in reversed(path.parents) if not d.exists()]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return made


def pct(xs, p):
    return speed_slice.pct(xs, p)


class Bench:
    def __init__(self, a):
        self.a = a
        base = Path(os.environ.get("TMPDIR", "/tmp"))
        self.work = Path(tempfile.mkdtemp(prefix="lust-tsedit-", dir=base))
        self.rng = random.Random(a.seed)
        self.states = []
        self.cold_n = 0
        self.daemon = None
        if a.slice:
            b = speed_slice.Bench(a.slice, {"no_driver": True, "no_rustc": True})
            self.tree = b.crate
            self.label = Path(a.slice).name
            self.target = {"name": "slice", "kind": "bin", "bin": "slice", "src": str(b.root), "manifest": str(b.crate / "Cargo.toml")}
            self.others = []
            self.tsrust_commit = b.manifest.get("tsrust_commit", "?")
            shutil.rmtree(b.work / "cache", ignore_errors=True)
        else:
            self.tree = self.work / "tree"
            shutil.copytree(Path(a.fork).resolve(), self.tree, ignore=shutil.ignore_patterns("target", ".git", ".lust-cache"))
            if a.clean:
                with tarfile.open(a.clean) as tf:
                    tf.extractall(self.tree, filter="data")
            ts = tsrust_full.targets(self.tree)
            by = {t["name"]: t for t in ts}
            if a.target not in by:
                sys.exit(f"tsedit: no target {a.target} in {a.fork} (have {', '.join(sorted(by))})")
            self.target = by[a.target]
            self.label = a.target + (" (clean fork)" if a.clean else "")
            self.others = [t for t in ts if t["kind"] == "lib" and t["name"] != a.target]
            head = Path(a.fork) / ".fork-head"
            self.tsrust_commit = head.read_text().strip() if head.exists() else subprocess.run(
                ["git", "-C", a.fork, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
        # The package manifest of the target and the workspace root's (manifest kinds).
        self.pkg_manifest = Path(self.target["manifest"]).resolve()
        root_manifest = self.tree / "Cargo.toml"
        self.ws_manifest = root_manifest if root_manifest.exists() and "[workspace]" in root_manifest.read_text() else self.pkg_manifest
        self.env = dict(os.environ, LUST_DAEMON_SOCKET=str(self.work / "s.sock"), LUST_CACHE_DIR=str(self.work / "cache"),
                        LUST_DAEMON_IDLE="7200s", CARGO_TARGET_DIR=str(self.work / "lust-cargo-target"), LUST_STATS="1")
        self.env.pop("LUST_NO_DAEMON", None)
        self.work.mkdir(parents=True, exist_ok=True)
        self.oracle_path = oracle_log.setup(self.env, str(self.work))
        self.cargo_env = dict(os.environ, CARGO_TARGET_DIR=str(self.work / "cargo-target"), CARGO_NET_OFFLINE="true")
        self.cargo_env.pop("RUSTFLAGS", None)

    # ---- processes

    def lust_cmd(self):
        return tsrust_full.lust_cmd(str(self.a.lust), "check", self.target, self.work) + ["--timings"]

    def run(self, cmd, env, timeout):
        t = time.perf_counter()
        try:
            r = subprocess.run(cmd, cwd=self.tree, env=env, capture_output=True, text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            return (time.perf_counter() - t) * 1000, None
        return (time.perf_counter() - t) * 1000, r

    def lust_warm(self):
        ms, r = self.run(self.lust_cmd(), self.env, WARM_TIMEOUT)
        if r is None:  # a hang: the daemon may be wedged, start a new one
            self.kill_daemon()
            self.start_daemon()
        return ms, r

    def lust_cold(self):
        self.cold_n += 1
        cache = self.work / f"cold-{self.cold_n}"
        # Its own Cargo target dir: a cold run's build script refresh must not touch the
        # daemon's, or the next warm request re-validates them (a bench artefact).
        env = dict(self.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(cache), CARGO_TARGET_DIR=str(self.work / "cold-cargo-target"))
        ms, r = self.run(self.lust_cmd(), env, COLD_TIMEOUT)
        shutil.rmtree(cache, ignore_errors=True)
        return ms, r

    def cargo(self):
        return self.run(tsrust_full.cargo_check_cmd(self.target, self.tree), self.cargo_env, COLD_TIMEOUT)

    def start_daemon(self):
        # The daemon log feeds the guard counts of the report (`guard_counts`).
        with open(self.work / "daemon.log", "a") as log:
            self.daemon = subprocess.Popen([str(self.a.lust), "daemon"], cwd=self.tree, env=self.env,
                                           stdout=subprocess.DEVNULL, stderr=log)
        for _ in range(800):
            if (self.work / "s.sock").exists():
                return
            time.sleep(0.025)
        sys.exit("tsedit: daemon did not start")

    def kill_daemon(self):
        if self.daemon:
            self.daemon.kill()
            self.daemon.wait()
            self.daemon = None
        for f in ("s.sock", "s.sock.lock"):
            (self.work / f).unlink(missing_ok=True)

    # ---- sites and setup

    def files(self, root):
        r = subprocess.run([str(prune.ITEMS), "--tree", str(root)], capture_output=True, text=True)
        return [Path(l).resolve() for l in r.stdout.splitlines() if Path(l).exists()]

    def compiled_files(self, cands):
        """The candidate files rustc compiles for the target: one cargo check with a
        `compile_error!` at the end of each (cfg'd-out and test-only files stay silent)."""
        saved = {p: p.read_text() for p in cands}
        for i, p in enumerate(cands):
            p.write_text(saved[p] + f'\ncompile_error!("lust-tsedit-probe {i}");\n')
        _, r = self.cargo()
        for p, t in saved.items():
            p.write_text(t)
        hit = set(int(m) for m in re.findall(r"lust-tsedit-probe (\d+)", r.stderr if r else ""))
        return [p for i, p in enumerate(cands) if i in hit]

    def body_ok(self, fn, extra=""):
        bad = set("xnca" + extra)
        return not (bad & set(fn["flags"])) and fn["cl"] > fn["ol"]

    def spread(self, items, n, key=lambda s: s["file"]):
        """Up to n items, one per file first (largest files first), seeded order within a file."""
        by = {}
        for it in items:
            by.setdefault(key(it), []).append(it)
        for v in by.values():
            self.rng.shuffle(v)
        files = sorted(by, key=lambda f: (-Path(f).stat().st_size, str(f)))
        out = []
        while len(out) < n and any(by.values()):
            for f in files:
                if by[f] and len(out) < n:
                    out.append(by[f].pop())
        return out

    def setup(self):
        root = Path(self.target["src"]).resolve()
        files = self.files(root)
        # Files another lib target or a build script also compiles stay out: an edit there
        # would change (or break) a dependency or the build script, not only this target.
        other = set()
        for t in self.others:
            other |= set(self.files(Path(t["src"]).resolve()))
        for b in self.tree.rglob("build.rs"):
            if "target" not in b.relative_to(self.tree).parts:
                other |= set(self.files(b.resolve()))
        cands = [p for p in files if p != root and p not in other]
        self.compiled = set(self.compiled_files(cands + [root]))
        print(f"tsedit: {self.label}: {len(files)} files in the module tree, {len(self.compiled)} compiled by rustc", flush=True)
        self.lines = sum(len(p.read_text().split("\n")) for p in self.compiled)
        kinds = self.a.kinds
        n = self.a.probes
        fns, derives = self.scan()
        setup_edits = {}
        if "sigx" in kinds:
            own = [f for f in fns if f["file"] != root and self.body_ok(f, "d")]
            callers = self.spread(own, max(2, n))
            for f in sorted(callers, key=lambda f: (f["file"], -f["ol"])):
                p = f["file"]
                setup_edits[p] = in_body(setup_edits.get(p, p.read_text()), f, "let _lust_c: u64 = crate::_lust_sigx(1).into();")
            setup_edits[root] = setup_edits.get(root, root.read_text()) + "\n#[allow(dead_code)]\npub(crate) fn _lust_sigx(x: u32) -> u64 { x as u64 }\n"
        if "oblig" in kinds:
            types = [d for d in derives if d["depth"] == 0 and not d["generic"] and "x" not in d["kind"]
                     and any(f["file"] == d["file"] and f["depth"] == 0 and self.body_ok(f) for f in fns)]
            self.oblig_types = self.spread(types, n)
            for p in {d["file"] for d in self.oblig_types}:
                helpers = "".join(f"\n#[allow(dead_code)]\nfn _lust_need_{k.lower()}<X: {path}>() {{}}" for k, path in NEED.items())
                setup_edits[p] = setup_edits.get(p, p.read_text()) + "\n" + helpers + "\n"
        for p, t in setup_edits.items():
            p.write_text(t)
        self.base = {p: p.read_text() for p in self.compiled}
        self.fns, self.derives = self.scan()
        if "oblig" in kinds:
            # Re-find the chosen types after the setup edits.
            want = {(d["file"], d["name"]) for d in self.oblig_types}
            self.oblig_types = [d for d in self.derives if (d["file"], d["name"]) in want and d["depth"] == 0]

    def scan(self):
        fns, derives = [], []
        for p in sorted(self.compiled):
            a, b = sites_of(p)
            fns += a
            derives += b
        return fns, derives

    # ---- probes

    def probes(self):
        """(kind, site label, [(variant, file, text)]) per probe; each probe ends on the base text."""
        n, out = self.a.probes, []
        fns = self.fns
        k = iter(range(1, 1_000_000))

        def lbl(f):
            return f"{os.path.relpath(f['file'], self.tree)}:{f['sig']} {f['name']}"

        for kind in self.a.kinds:
            if kind == "body":
                for f in self.spread([f for f in fns if self.body_ok(f)], n):
                    t = self.base[f["file"]]
                    i = next(k)
                    out.append((kind, lbl(f), [("accept", f["file"], in_body(t, f, f"let _lust_e: u32 = {i};")),
                                               ("reject", f["file"], in_body(t, f, f'let _lust_e: u32 = "{i}";'))]))
            elif kind == "blockbody":
                for f in self.spread([f for f in fns if self.body_ok(f) and "b" in f["flags"]], n):
                    t = self.base[f["file"]]
                    i = next(k)
                    out.append((kind, lbl(f), [("accept", f["file"], in_body(t, f, f"let _lust_e: u32 = {i};")),
                                               ("reject", f["file"], in_body(t, f, f'let _lust_e: u32 = "{i}";'))]))
            elif kind == "macbody":
                pure = ("matches!(", "write!(", "writeln!(", "debug_assert")

                def calls_pure(f):
                    lines = self.base[f["file"]].splitlines()[f["ol"] - 1:f["cl"]]
                    return any(m in ln for ln in lines for m in pure)
                for f in self.spread([f for f in fns if self.body_ok(f) and calls_pure(f)], n):
                    t = self.base[f["file"]]
                    i = next(k)
                    out.append((kind, lbl(f), [("accept", f["file"], in_body(t, f, f"let _lust_e: u32 = {i};")),
                                               ("reject", f["file"], in_body(t, f, f'let _lust_e: u32 = "{i}";'))]))
            elif kind == "infer":
                for f in self.spread([f for f in fns if self.body_ok(f)], n):
                    t = self.base[f["file"]]
                    i = next(k)
                    s = "let _lust_i = [{v}; 2]; let _lust_s: {ty} = _lust_i.iter().sum();"
                    out.append((kind, lbl(f), [("accept", f["file"], in_body(t, f, s.format(v=i, ty="u64"))),
                                               ("accept", f["file"], in_body(t, f, s.format(v=i, ty="u32"))),
                                               ("reject", f["file"], in_body(t, f, s.format(v=f"{i}u8", ty="u64")))]))
            elif kind == "sig":
                for f in self.spread([f for f in fns if self.body_ok(f, "tdg")], n):
                    t = self.base[f["file"]]
                    out.append((kind, lbl(f), [("accept", f["file"], add_where(t, f, "u32: Copy")),
                                               ("reject", f["file"], add_where(t, f, "String: Copy")),
                                               ("reject", f["file"], add_param(t, f))]))
            elif kind == "sigx":
                root = Path(self.target["src"]).resolve()
                t = self.base[root]
                old = "pub(crate) fn _lust_sigx(x: u32) -> u64 { x as u64 }"
                forms = [("accept", "pub(crate) fn _lust_sigx(x: u32) -> u32 { x }"),
                         ("accept", "pub(crate) fn _lust_sigx(x: u8) -> u16 { x as u16 }"),
                         ("reject", "pub(crate) fn _lust_sigx(x: u32) -> i64 { x as i64 }")]
                for i in range(max(1, n // 2)):
                    out.append((kind, f"{os.path.relpath(root, self.tree)} _lust_sigx #{i + 1}",
                                [(v, root, t.replace(old, new)) for v, new in forms]))
            elif kind == "oblig":
                for d in self.oblig_types:
                    t = self.base[d["file"]]
                    users = [f for f in fns if f["file"] == d["file"] and f["depth"] == 0 and self.body_ok(f)]
                    f = self.rng.choice(users)
                    have = [x for x in d["traits"] if x in NEED]
                    lack = [x for x in NEED if x not in d["traits"]]
                    states = []
                    if have:
                        states.append(("accept", d["file"], in_body(t, f, f"_lust_need_{self.rng.choice(have).lower()}::<{d['name']}>();")))
                    if lack:
                        states.append(("reject", d["file"], in_body(t, f, f"_lust_need_{self.rng.choice(lack).lower()}::<{d['name']}>();")))
                    out.append((kind, f"{os.path.relpath(d['file'], self.tree)}:{d['line']} {d['name']} in {f['name']}", states))
            elif kind == "derive":
                cands = [d for d in self.derives if "x" not in d["kind"] and len(d["traits"]) >= 2]
                for d in self.spread(cands, n):
                    t = self.base[d["file"]]
                    prefer = [x for x in ("Clone", "Debug", "PartialEq", "Default", "Hash", "Copy") if x in d["traits"]]
                    drop = self.rng.choice(prefer or d["traits"])
                    lines = t.split("\n")
                    line = lines[d["line"] - 1]
                    m = re.search(r"#\[derive\(([^)]*)\)\]", line)
                    if not m:
                        continue
                    keep = [x.strip() for x in m.group(1).split(",") if x.strip() and x.strip().split("::")[-1] != drop.split("::")[-1]]
                    lines[d["line"] - 1] = line[:m.start(1)] + ", ".join(keep) + line[m.end(1):]
                    out.append((kind, f"{os.path.relpath(d['file'], self.tree)}:{d['line']} {d['name']} -{drop}",
                                [("edit", d["file"], "\n".join(lines))]))
            elif kind == "generic":
                texts = {p: t for p, t in self.base.items()}
                used = [f for f in fns if "g" in f["flags"] and self.body_ok(f) and any(
                    p != f["file"] and re.search(rf"\b{re.escape(f['name'])}\b", tx) for p, tx in texts.items())]
                for f in self.spread(used, n):
                    t = self.base[f["file"]]
                    T = f["tparams"][0]
                    out.append((kind, lbl(f), [("accept", f["file"], in_body(t, f, f"let _lust_g: Option<&{T}> = None;")),
                                               ("reject", f["file"], in_body(t, f, f"let _lust_g: Option<&{T}> = None; let _lust_h: Option<{T}> = _lust_g.cloned();"))]))
            elif kind == "rpit":
                cands = [f for f in fns if "r" in f["flags"] and self.body_ok(f, "t")]
                for f in self.spread(cands, n):
                    t = self.base[f["file"]]
                    states = [("accept", f["file"], in_body(t, f, "let _lust_e: u32 = 1;"))]
                    tail = ret_text(t, f).rsplit("impl", 1)[-1]
                    if not re.search(r"\b(Fn|FnMut|FnOnce)\s*\(|\bdyn\b", tail):
                        states.append(("accept", f["file"], insert_at(t, f["rl"], f["rc"] - 1, " + Sized")))
                        states.append(("reject", f["file"], insert_at(t, f["rl"], f["rc"] - 1, " + Copy")))
                    out.append((kind, lbl(f), states))
            elif kind in MANIFEST_KINDS:
                out += self.manifest_probes(kind, n, lbl)
        return out

    def manifest_probes(self, kind, n, lbl):
        """Probes of one manifest kind (module doc). A state is (variant, {path: text or None}, None):
        the whole text of every file it changes (None deletes the file), on top of the last state."""
        pm, wm = self.pkg_manifest, self.ws_manifest
        ptext, wtext = pm.read_text(), wm.read_text()
        pdir = pm.parent
        bad = 'let _lust_m: u32 = "lust";'
        sites = self.spread([f for f in self.fns if self.body_ok(f)], n)
        out = []
        if kind == "mfeat":
            lt = self.tree / ".lust.toml"
            exact = lt.exists() and re.search(r"^dev-features\s*=", lt.read_text(), re.M)
            for f in sites:
                t = self.base[f["file"]]
                decl = toml_add(ptext, "features", "_lust_f = []")
                on = {lt: toml_list_add(lt.read_text(), "", "dev-features", "_lust_f")} if exact else {pm: toml_list_add(decl, "features", "default", "_lust_f")}
                off = {lt: lt.read_text()} if exact else {pm: decl}
                out.append((kind, lbl(f), [("accept", {pm: decl, f["file"]: in_body(t, f, f'#[cfg(feature = "_lust_f")] {bad}')}, None),
                                           ("reject", on, None), ("accept", off, None)]))
        elif kind == "medition":
            new = re.sub(r'^edition(\.workspace\s*=\s*true|\s*=\s*"2024")', 'edition = "2021"', ptext, count=1, flags=re.M)
            if new != ptext:
                out.append((kind, f"{os.path.relpath(pm, self.tree)} edition 2021", [("edit", {pm: new}, None)]))
        elif kind == "mlints":
            inherit = any(re.match(r"\s*workspace\s*=\s*true", l) for l in toml_section(ptext, "lints"))
            if inherit or not toml_section(ptext, "lints"):
                path, text, sec = (wm, wtext, "workspace.lints.rust") if inherit else (pm, ptext, "lints.rust")
                out.append((kind, f"{os.path.relpath(path, self.tree)} [{sec}]",
                            [("edit", {path: toml_add(text, sec, 'non_ascii_idents = "deny"')}, None),
                             ("accept", {path: toml_add(text, sec, 'non_ascii_idents = "warn"')}, None)]))
        elif kind == "mprofile" and not any(l.strip().startswith("debug-assertions") for l in toml_section(wtext, "profile.dev")):
            for f in sites:
                t = self.base[f["file"]]
                out.append((kind, lbl(f), [("reject", {f["file"]: in_body(t, f, f"#[cfg(debug_assertions)] {bad}")}, None),
                                           ("accept", {wm: toml_add(wtext, "profile.dev", "debug-assertions = false")}, None),
                                           ("reject", {wm: wtext}, None)]))
        elif kind == "mdeps":
            for l in toml_section(ptext, "dependencies"):
                m = re.match(r'\s*([\w-]+)\s*=\s*"=(\d+)\.(\d+)\.(\d+)"\s*$', l)
                if m:
                    name = m.group(1)
                    loose = ptext.replace(l, f'{name} = "{m.group(2)}.{m.group(3)}"', 1)
                    out.append((kind, f"{os.path.relpath(pm, self.tree)} {name}",
                                [("accept", {pm: loose}, None), ("reject", {pm: ptext.replace(l, f'{name} = "=99.0.0"', 1)}, None)]))
                    if len(out) >= n:
                        break
        elif kind == "mtarget":
            if not (pdir / "build.rs").exists():
                script = 'fn main() {\n    println!("cargo::rustc-check-cfg=cfg(_lust_bs)");\n    println!("cargo::rustc-cfg=_lust_bs");\n}\n'
                for f in sites:
                    t = self.base[f["file"]]
                    out.append((kind, lbl(f), [("accept", {pdir / "build.rs": script}, None),
                                               ("reject", {f["file"]: in_body(t, f, f"#[cfg(_lust_bs)] {bad}")}, None),
                                               ("edit", {pdir / "build.rs": None}, None)]))
            else:
                main = "fn main() {}\n"
                out.append((kind, f"{os.path.relpath(pdir, self.tree)} new targets",
                            [("accept", {pdir / "src/bin/_lust_x.rs": main}, None), ("accept", {pdir / "examples/_lust_x.rs": main}, None)]))
        return out

    # ---- states

    def measure(self, kind, site, variant, step):
        lms, lr = self.lust_warm()
        cms, cr = self.cargo()
        rec = {"kind": kind, "site": site, "variant": variant, "step": step, "lust_ms": round(lms, 2), "cargo_ms": round(cms, 2)}
        if lr is None:
            rec["class"] = "hang (warm)"
        elif cr is None:
            rec["class"] = "cargo timeout"
        else:
            lok, cok = lr.returncode == 0, cr.returncode == 0
            visits = speed_slice.parse_visits(lr.stderr)
            pk = speed_slice.classify_request(visits, speed_slice.parse_timings(lr.stderr))
            rec.update(lust_ok=lok, cargo_ok=cok, phases=speed_slice.parse_timings(lr.stderr), visits=visits,
                       path=f"{pk['patch']['status']}:{pk['patch']['reason'] or ''} kept={pk['kept']['status']}:{pk['kept']['reason'] or ''}")
            rec["class"] = "match" if lok == cok else ("FALSE ACCEPT" if lok else "false reject")
            if not cok:
                rec["cargo_error"] = next((l for l in cr.stderr.splitlines() if ": error" in l), cr.stderr.strip()[-300:])[:300]
            if not lok:
                rec["lust_error"] = next((l for l in lr.stdout.splitlines() if '"severity":"error"' in l), lr.stderr.strip()[-300:])[:300]
            if not self.a.no_cold:
                _, c = self.lust_cold()
                if c is None:
                    rec["cold"] = "hang"
                elif (c.returncode, c.stdout, speed_slice.strip_noise(c.stderr)) != (lr.returncode, lr.stdout, speed_slice.strip_noise(lr.stderr)):
                    rec["cold"] = "WARM != COLD"
                    rec["cold_detail"] = {"warm_rc": lr.returncode, "cold_rc": c.returncode,
                                          "warm": (lr.stdout + speed_slice.strip_noise(lr.stderr))[-1200:],
                                          "cold": (c.stdout + speed_slice.strip_noise(c.stderr))[-1200:]}
                else:
                    rec["cold"] = "same"
        self.states.append(rec)
        bad = rec["class"] not in ("match",) or rec.get("cold") not in (None, "same")
        if bad or self.a.verbose:
            print(f"tsedit: {kind} {variant} {site}: lust {lms:.0f} ms, cargo {cms:.0f} ms, {rec['class']}, cold {rec.get('cold')}, "
                  f"{rec.get('path', '')}", flush=True)
        return rec

    def go(self):
        prune.build_items_tool()
        t0 = time.time()
        self.setup()
        _, r = self.cargo()
        if r is None or r.returncode != 0:
            sys.exit(f"tsedit: cargo check rejects the base tree:\n{(r.stderr if r else 'timeout')[-2000:]}")
        _, c = self.lust_cold()
        if c is None or c.returncode != 0:
            sys.exit(f"tsedit: lust rejects the base tree:\n{(c.stdout + c.stderr if c else 'hang')[-2000:]}")
        self.start_daemon()
        first_ms, _ = self.lust_warm()
        noop = [self.lust_warm()[0] for _ in range(3)]
        print(f"tsedit: {self.label}: {self.lines} lines; daemon first check {first_ms:.0f} ms, no-op {statistics.median(noop):.1f} ms", flush=True)
        plan = self.probes()
        for kind, site, states in plan:
            # A state names one file (path, text) or several ({path: text or None}, None).
            old, made = {}, []
            for i, (variant, path, text) in enumerate(states):
                for p, t in (path if isinstance(path, dict) else {path: text}).items():
                    if p not in old:
                        old[p] = p.read_text() if p.exists() else None
                    made += write_file(p, t)
                self.measure(kind, site, variant, i + 1)
            for p, t in old.items():
                write_file(p, t)
            for d in reversed(made):
                if d.exists() and not any(d.iterdir()):
                    d.rmdir()
            self.measure(kind, site, "revert", len(states) + 1)
        self.kill_daemon()
        return self.report(first_ms, noop, time.time() - t0)

    # ---- report

    def report(self, first_ms, noop, secs):
        rows = []
        for kind in self.a.kinds:
            st = [s for s in self.states if s["kind"] == kind]
            ok = [s for s in st if s.get("cargo_ok")]
            err = [s for s in st if s.get("cargo_ok") is False]
            row = {"kind": kind, "states": len(st), "ok_states": len(ok), "reject_states": len(err),
                   "false_accepts": sum(s["class"] == "FALSE ACCEPT" for s in st),
                   "false_rejects": sum(s["class"] == "false reject" for s in st),
                   "warm_ne_cold": sum(s.get("cold") == "WARM != COLD" for s in st),
                   "hangs": sum("hang" in s["class"] or s.get("cold") == "hang" for s in st)}
            if ok:
                lm, cm = [s["lust_ms"] for s in ok], [s["cargo_ms"] for s in ok]
                row.update(lust=speed_slice.summary(lm), cargo=speed_slice.summary(cm))
                row["ratio"] = speed_slice.ratio(row["cargo"]["median"], row["lust"]["median"])
                row["ratio_p95"] = speed_slice.ratio(row["cargo"]["p95"], row["lust"]["p95"])
            if err:
                row["reject_lust"] = speed_slice.summary([s["lust_ms"] for s in err])
                row["reject_cargo"] = speed_slice.summary([s["cargo_ms"] for s in err])
            rows.append(row)
        lust_commit = os.environ.get("LUST_COMMIT") or subprocess.run(
            ["git", "-C", str(ROOT), "describe", "--always", "--dirty"], capture_output=True, text=True).stdout.strip()
        res = {"tree": self.label, "lines": self.lines, "files": len(self.compiled), "tsrust_commit": self.tsrust_commit,
               "lust_commit": lust_commit, "rustc": subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip(),
               "host": {"nproc": os.cpu_count(), "load_at_end": os.getloadavg()[0]}, "seed": self.a.seed, "probes_per_kind": self.a.probes,
               "daemon_first_ms": round(first_ms, 1), "noop_ms": round(statistics.median(noop), 2), "secs": round(secs),
               "rows": rows, "states": self.states}
        # Under LUST_CHECK_ITEMPATCH=1: the oracle's checks and failures
        # (a target with no check fails, see scripts/oracle_log.py).
        ok, fails = oracle_log.collect(self.oracle_path)
        res["oracle"] = {"on": oracle_log.on(), "checks": ok, "problems": oracle_log.problems(ok, fails, self.label)}
        res["guard"] = guard_counts(self.work / "daemon.log")
        if not self.a.keep:
            shutil.rmtree(self.work, ignore_errors=True)
        return res


def guard_counts(log):
    """The daemon's background guard checks (src/daemon.rs): started, and
    done as match, stale (an input changed during the check), cancelled (the
    daemon saw an input change and stopped it), or mismatch.
    `guard_lines` counts the loud `guard:` lines; any one is a bug or a
    false alarm to look at, and a mismatch costs the next request a full
    fresh compile."""
    text = log.read_text() if log.exists() else ""
    done = lambda kind: len(re.findall(rf"^guard-check: done \d+ {kind}", text, re.M))  # noqa: E731
    return {"started": len(re.findall(r"^guard-check: start ", text, re.M)), "match": done("match"), "stale": done("stale"),
            "cancelled": done("cancelled"), "mismatch": done("mismatch"), "guard_lines": len(re.findall(r"^guard:", text, re.M))}


def print_table(res):
    print(f"\n{res['tree']}: {res['lines']} lines in {res['files']} files, ts-rust {res['tsrust_commit'][:9]}, lust {res['lust_commit']}, "
          f"{res['rustc']}, {res['host']['nproc']} cpus; daemon first {res['daemon_first_ms']:.0f} ms, no-op {res['noop_ms']} ms")
    g = res.get("guard")
    if g:
        print(f"guard checks: {g['started']} started, {g['match']} match, {g['stale']} stale, {g.get('cancelled', 0)} cancelled, {g['mismatch']} mismatch, "
              f"{g['guard_lines']} guard: lines")
    print(f"{'kind':<8} {'ok':>4} {'rej':>4} {'lust med':>9} {'p95':>8} {'cargo med':>10} {'p95':>8} {'ratio':>8} {'p95':>8}"
          f"  {'FA':>3} {'FR':>3} {'w!=c':>4} {'hang':>4}")
    for r in res["rows"]:
        l, c = r.get("lust"), r.get("cargo")
        sp = (f"{l['median']:>9.1f} {l['p95']:>8.1f} {c['median']:>10.1f} {c['p95']:>8.1f} {str(r['ratio']) + 'x':>8} {str(r['ratio_p95']) + 'x':>8}"
              if l else f"{'-':>9} {'-':>8} {'-':>10} {'-':>8} {'-':>8} {'-':>8}")
        print(f"{r['kind']:<8} {r['ok_states']:>4} {r['reject_states']:>4} {sp}  {r['false_accepts']:>3} {r['false_rejects']:>3} "
              f"{r['warm_ne_cold']:>4} {r['hangs']:>4}")


def summary_line(res):
    rs = res["rows"]
    kinds = " ".join(f"{r['kind']} {r['ratio']}x/{r['ratio_p95']}x" for r in rs if r.get("ratio"))
    tot = lambda k: sum(r[k] for r in rs)  # noqa: E731
    o = res.get("oracle", {})
    oracle = f", oracle {o['checks']} checks {len(o['problems'])} failures" if o.get("on") else ""
    return (f"tsedit: {res['tree']} {res['lines']} lines: {kinds}; {len(res['states'])} states, {tot('false_accepts')} false accepts, "
            f"{tot('false_rejects')} false rejects, {tot('warm_ne_cold')} warm!=cold, {tot('hangs')} hangs{oracle}")


def census(a):
    """The clean fork for --clean auto: tsrust_deep.py --rustc --save-clean over the targets."""
    out = Path(a.out).resolve() / "census"
    out.mkdir(parents=True, exist_ok=True)
    r = subprocess.run([sys.executable, str(HERE / "tsrust_deep.py"), "--fork", a.fork, "--lust", str(a.lust), "--work", str(out),
                        "--only", a.census_only or a.target, "--rustc", "--save-clean", "clean.tgz"])
    if r.returncode != 0 or not (out / "clean.tgz").exists():
        sys.exit("tsedit: the census did not write clean.tgz")
    shutil.rmtree(out / "fork", ignore_errors=True)
    return str(out / "clean.tgz")


def merge_shards(results):
    """One result per tree: shards of the same tree (runs with other `--kinds`) join, rows in
    KINDS order; a later shard's row of a kind wins."""
    by = {}
    for res in results:
        m = by.setdefault(res["tree"], dict(res, rows=[], states=[]))
        rows = {r["kind"]: r for r in m["rows"]}
        rows.update({r["kind"]: r for r in res["rows"]})
        m["rows"] = [rows[k] for k in ALL_KINDS if k in rows]
        m["states"] = m["states"] + res["states"]
    return list(by.values())


def render_md(results):
    """The docs/speed.md section for tsedit result JSONs: one table per tree."""
    out = []
    for res in merge_shards(results):
        out += [f"### `{res['tree']}`: {res['lines']:,} lines, {res['files']} files", "",
                f"ts-rust `{res['tsrust_commit'][:9]}`, lust `{res['lust_commit']}`, {res['rustc']}, {res['host']['nproc']} vCPU, "
                f"{res['probes_per_kind']} probes per kind, {len(res['states'])} states. Daemon first check {res['daemon_first_ms']:.0f} ms, no-op {res['noop_ms']} ms.", "",
                "| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |",
                "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for r in res["rows"]:
            l, c = r.get("lust"), r.get("cargo")
            sp = (f"{l['median']:.1f} ms | {l['p95']:.1f} | {c['median']:.0f} ms | {c['p95']:.0f} | **{r['ratio']}x** | **{r['ratio_p95']}x**"
                  if l else "- | - | - | - | - | -")
            out.append(f"| {r['kind']} | {r['ok_states']} | {sp} | {r['reject_states']} | {r['false_accepts']} | {r['warm_ne_cold']} |")
        out.append("")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--fork")
    ap.add_argument("--target", help="target name, or a comma list (one bench per target, in order)")
    ap.add_argument("--clean", help="tar.gz from tsrust_deep.py --save-clean, unpacked over the fork copy; "
                    "`auto` runs that census first (into --out)")
    ap.add_argument("--census-only", help="targets the --clean auto census prunes (default: --target)")
    ap.add_argument("--slice")
    ap.add_argument("--lust", default=str(ROOT / "target/release/lust"))
    ap.add_argument("--kinds", default=",".join(KINDS), help="comma list; `manifest` adds every manifest kind")
    ap.add_argument("--probes", type=int, default=8, help="probes per kind (default 8)")
    ap.add_argument("--quick", action="store_true", help="2 probes per kind")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--no-cold", action="store_true", help="skip the cold lust check of every state")
    ap.add_argument("--json", help="result file (a list when there are several targets)")
    ap.add_argument("--out", default=".", help="dir for --clean auto's census and per-target JSONs")
    ap.add_argument("--keep", action="store_true", help="keep the work dir")
    ap.add_argument("--verbose", action="store_true", help="print every state")
    ap.add_argument("--render", nargs="+", metavar=("SPEED_MD", "JSON"), help="write the tsedit tables of these JSONs into docs/speed.md")
    a = ap.parse_args()
    if a.render:
        md, files = Path(a.render[0]), a.render[1:]
        res = [r for f in files for r in (lambda d: d if isinstance(d, list) else [d])(json.loads(Path(f).read_text()))]
        mark_a, mark_b = "<!-- tsedit-table:start -->", "<!-- tsedit-table:end -->"
        text = md.read_text()
        head, rest = text.split(mark_a, 1)
        md.write_text(head + mark_a + "\n" + render_md(res) + mark_b + rest.split(mark_b, 1)[1])
        return
    if bool(a.slice) == bool(a.fork and a.target):
        sys.exit("tsedit: give --slice DIR or --fork DIR --target NAME")
    a.kinds = [m for k in a.kinds.split(",") if k for m in (MANIFEST_KINDS if k == "manifest" else [k])]
    if bad := [k for k in a.kinds if k not in ALL_KINDS]:
        sys.exit(f"tsedit: unknown kinds {bad} (have {ALL_KINDS}, manifest)")
    if a.quick:
        a.probes = 2
    a.lust = Path(a.lust).resolve()
    if a.clean == "auto":
        a.clean = census(a)
    results, lines = [], []
    for t in (a.target.split(",") if a.target else [None]):
        a1 = argparse.Namespace(**{**vars(a), "target": t})
        res = Bench(a1).go()
        print_table(res)
        results.append(res)
        lines.append(summary_line(res))
        if a.target and "," in a.target:
            Path(a.out, f"tsedit-{t.replace(':', '-')}.json").write_text(json.dumps(res, indent=1) + "\n")
    if a.json:
        Path(a.json).write_text(json.dumps(results if len(results) > 1 else results[0], indent=1) + "\n")
        print(f"json: {a.json}")
    for line in lines:
        print(line)
    rs = [r for res in results for r in res["rows"]]
    oracle_bad = [p for res in results for p in res.get("oracle", {}).get("problems", [])]
    for p in oracle_bad[:20]:
        print(f"ORACLE {p}")
    sys.exit(1 if oracle_bad or any(r["false_accepts"] or r["warm_ne_cold"] or r["hangs"] for r in rs) else 0)


if __name__ == "__main__":
    main()
