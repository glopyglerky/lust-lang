#!/usr/bin/env python3
"""Deep census of the ts-rust fork: how many lines each lust root cause keeps out.

`lust check` stops a crate at its first failing stage (often one error), so a plain census
shows only the front. This works on a scratch copy of the fork: check a target, remove the
innermost item that holds each error (bench/slices/prune.py's item tool and rules), and
repeat until the target checks or no error can be located. Every removal is charged to a
root cause: its own (code, normalized message), or, when its error names an item an earlier
removal took away, that item's root cause (a cascade).

The copy is scratch: nothing here is a fork change. Fix the causes in lust or in the fork,
then run the plain census (scripts/tsrust-full.sh).

Usage: tsrust_deep.py --fork DIR --lust PATH --work DIR [--only a,b] [--max-rounds N]
                       [--rustc] [--save-clean FILE.tgz]
Targets default to the fork's library crates in dependency order (tsrust_full.census_libs). Output: <work>/deep.json and a ranked table on stdout.

--rustc makes a clean fork that BOTH compilers accept (scripts/tsedit.py edits it): when lust
accepts a target, `cargo check` it with lust's dev features too, remove the items that hold
rustc's errors (cause `rustc:<code>`), and go on. A rustc-only error on code lust accepts is a
census artefact (an item a removal left half used) or a lust false accept: the run prints them
as `tsrust-deep: rustc-only` lines. All targets then run again until one pass removes nothing,
because a later target's removals can break an earlier one (shared files).
--save-clean writes the files of the pruned copy that differ from the fork as a tar.gz (unpack
it over a fork tree to get the clean fork).
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / "bench/slices"))
import prune  # noqa: E402  (bench/slices/prune.py: items_of, prune_round, located)
import tsrust_full  # noqa: E402

def errors(lust, t, work, env):
    """Located errors of one lust check: (file, line, code, message). Line 0 when the span is
    not where the file on disk says (lust std source shown under a user file)."""
    r = subprocess.run(tsrust_full.lust_cmd(lust, "check", t, work), cwd=work, env=env, capture_output=True, text=True, timeout=3600)
    out = []
    for line in r.stdout.splitlines():
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("kind") != "diagnostic" or d.get("severity") != "error":
            continue
        sp = d.get("span") or {}
        f = d.get("file") or ""
        ln = sp.get("line", 0) if f and prune.located(f, sp) else 0
        out.append((f, ln, d.get("code", "?"), d.get("message", "")))
    return r.returncode, out, r.stderr


def rustc_errors(t, work, copy):
    """`cargo check` of one target in the copy: (rc, located errors as (file, line, code, msg))."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(work / "rustc-target"))
    r = subprocess.run(tsrust_full.cargo_check_cmd(t, copy), cwd=copy, env=env, capture_output=True, text=True, timeout=3600)
    out = []
    for line in r.stderr.splitlines():
        m = re.match(r"^(.+?\.rs):(\d+):(\d+): error(\[E\d+\])?: (.*)$", line)
        if m:
            out.append((str((copy / m.group(1)).resolve()), int(m.group(2)), (m.group(4) or "").strip("[]") or "E????", m.group(5)))
    return r.returncode, out, r.stderr


def save_clean(fork, copy, dest):
    """tar.gz of the copy's files that differ from the fork (paths relative to the fork root)."""
    import tarfile

    changed = []
    for p in sorted(copy.rglob("*.rs")):
        rel = p.relative_to(copy)
        if "target" in rel.parts:
            continue
        orig = fork / rel
        if not orig.exists() or orig.read_bytes() != p.read_bytes():
            changed.append(rel)
    with tarfile.open(dest, "w:gz") as tf:
        for rel in changed:
            tf.add(copy / rel, arcname=str(rel))
    print(f"tsrust-deep: clean fork: {len(changed)} changed files in {dest}", flush=True)


def in_item(err, cache):
    """True when a removable item holds the error's line (else prune.py would search by name)."""
    f = str(Path(err[0]).resolve())
    return any(it[0] <= err[1] <= it[1] for it in prune.items_of(f, cache))


def remove_users(copy, names, cause_of_name, removed, target):
    """Remove every innermost item of the copy whose text names one of `names` as a word.
    Each removal is a cascade of that name's root cause. Returns the number removed."""
    pat = re.compile(r"\b(" + "|".join(map(re.escape, sorted(names))) + r")\b")
    n = 0
    for p in sorted(copy.rglob("*.rs")):
        # Only files a lib or bin target compiles: no integration tests, benches or examples.
        if {"target", "tests", "benches", "examples"} & set(p.relative_to(copy).parts):
            continue
        lines = p.read_text().split("\n")
        items = prune.items_of(str(p), {})
        hits = {}
        for i, line in enumerate(lines, 1):
            m = pat.search(line.split("//")[0])
            if not m:
                continue
            inner = [it for it in items if it[0] <= i <= it[1]]
            if inner:
                it = max(inner, key=lambda t: (t[2], -(t[1] - t[0])))
                hits.setdefault((it[0], it[1]), (it, m.group(1)))
        spans = sorted(hits)
        outer = []
        for sp in spans:
            if outer and sp[0] >= outer[-1][0] and sp[1] <= outer[-1][1]:
                continue
            outer.append(sp)
        for sp in sorted(outer, reverse=True):
            it, nm = hits[sp]
            del lines[sp[0] - 1:sp[1]]
            removed.append({"file": os.path.relpath(p, copy), "kind": it[3], "name": it[4], "lines": sp[1] - sp[0] + 1,
                            "by": "users", "code": "USES", "message": f"uses removed `{nm}`",
                            "cause": list(cause_of_name[nm]), "target": target})
            cause_of_name.setdefault(it[4], cause_of_name[nm])
            n += 1
        if outer:
            p.write_text("\n".join(lines))
    return n


def names_in(msg):
    """Item names an error message points at: the last path segment of each `...` text."""
    out = set()
    for q in re.findall(r"`([^`]+)`", msg):
        seg = re.split(r"::", q.strip("&* "))[-1]
        m = re.match(r"[A-Za-z_]\w*", seg)
        if m:
            out.add(m.group(0))
    return out


def impl_names(msg):
    """The impl key (prune.impl_keys) an error about a missing method or impl points at: a
    removed impl block's `Type::method` or `impl Trait for Type`."""
    m = re.search(r"no method `(\w+)` on type `([^`]+)`", msg)
    if m:
        return [f"{prune.short_type(m.group(2))}::{m.group(1)}"]
    m = re.search(r"no impl of `([^`]+)` for `([^`]+)`", msg)
    return [f"impl {prune.short_type(m.group(1))} for {prune.short_type(m.group(2))}"] if m else []


# A call, macro, turbofish or path through a name: `f(`, `.m(`, `m!`, `f::<`, `T::`.
USED = re.compile(r"\b([A-Za-z_]\w*)\s*(?:\(|!|::)")


def unknown_type_parents(located, copy, cause_of_name):
    """For each error whose type lust could not know ({projection}, {error}) because the census
    removed what produced the value: the cause of the nearest removed name the code calls or
    paths through, on the error line first, then on the lines above it in its item.
    Keyed (file relative to the copy, message as prune.py stores it)."""
    out, cache = {}, {}
    for f, ln, _code, msg in located:
        if "{projection}" not in msg and "{error}" not in msg:
            continue
        item = [it for it in prune.items_of(str(Path(f).resolve()), cache) if it[0] <= ln <= it[1]]
        top = max(item, key=lambda it: it[0])[0] if item else ln
        lines = Path(f).read_text().split("\n")
        for i in range(ln, top - 1, -1):
            hit = next((n for n in USED.findall(lines[i - 1].split("//")[0]) if n in cause_of_name and n not in ("main", "Self")), None)
            if hit:
                out[(os.path.relpath(Path(f).resolve(), copy), msg[:160])] = cause_of_name[hit]
                break
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fork", required=True, type=Path)
    ap.add_argument("--lust", required=True)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--only", default="")
    ap.add_argument("--max-rounds", type=int, default=200)
    ap.add_argument("--rustc", action="store_true", help="also prune until cargo check accepts each target")
    ap.add_argument("--save-clean", default="", help="write the changed files of the pruned copy to this tar.gz (relative to --work)")
    a = ap.parse_args()
    prune.build_items_tool()
    work = a.work.resolve()
    copy = work / "fork"
    if copy.exists():
        shutil.rmtree(copy)
    shutil.copytree(a.fork.resolve(), copy, ignore=shutil.ignore_patterns("target", ".git", ".lust-cache"))
    env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(work / "lust-cache"), CARGO_TARGET_DIR=str(work / "cargo-target"))
    by_name = {t["name"]: t for t in tsrust_full.targets(copy)}
    a.only = a.only or tsrust_full.census_libs(copy)
    removed, cause_of_name, status = [], {}, {}
    # One check of every target on the unpruned copy first: a target that needs a build script
    # record lust has not stored yet (the tsgo bin) gets it through `cargo check` now. After the
    # pruning, rustc can reject the copy (a removed item whose only user is cfg'd out for lust).
    for name in a.only.split(","):
        errors(a.lust, by_name[name], work, env)
    for pass_no in range(1, (4 if a.rustc else 1) + 1):
        pass_start = len(removed)
        one_pass(a, by_name, work, copy, env, removed, cause_of_name, status, pass_no)
        if len(removed) == pass_start:
            break
    if a.save_clean:
        save_clean(a.fork.resolve(), copy, work / a.save_clean)
    report(removed, status, copy, work)


def one_pass(a, by_name, work, copy, env, removed, cause_of_name, status, pass_no):
    """One run over the targets: prune each until it checks (or stops). Rounds after the first
    pass only start when the target fails again."""
    for name in a.only.split(","):
        t = by_name[name]
        for rnd in range(1, a.max_rounds + 1):
            rc, errs, stderr = errors(a.lust, t, work, env)
            tool = "lust"
            if rc == 0 and a.rustc:
                rc, errs, stderr = rustc_errors(t, work, copy)
                tool = "rustc"
                for e in errs:
                    print(f"tsrust-deep: rustc-only {name}: {os.path.relpath(e[0], copy)}:{e[1]} {e[2]} {e[3][:160]}", flush=True)
                if rc != 0 and not errs:
                    status[name] = f"stopped: cargo check exit {rc} with no located error: {stderr.strip()[-300:]}"
                    break
            if rc == 0:
                status[name] = f"ok after {rnd - 1} rounds" + (f" (pass {pass_no})" if pass_no > 1 else "")
                break
            if not errs:
                status[name] = f"stopped: exit {rc} with no diagnostic: {tsrust_full.crash_reason({'stderr': stderr, 'rc': rc})}"
                break
            # Only errors with a span on disk inside an item: prune.py's fallback for the others
            # (remove every item whose text names the error's one `name`) takes out unrelated code.
            # Each round's errors are kept in <work>/round-<target>-<n>.json.
            (work / f"round-{name}-{rnd}.json").write_text(json.dumps(errs))
            cache = {}
            located = [e for e in errs if e[1] > 0 and Path(e[0]).resolve().is_relative_to(copy) and in_item(e, cache)]
            if not located:
                # Errors with no span on disk that name an item an earlier round removed (macro
                # output that used it): remove the items that mention that name, as its cascade.
                # `main` is no item name another item uses (every bin has one).
                gone = {n for e in errs for n in names_in(e[3]) if n in cause_of_name and n != "main"}
                if not gone or not remove_users(copy, gone, cause_of_name, removed, name):
                    status[name] = f"stopped: no located error: {errs[0][2]} {errs[0][3][:160]}"
                    break
                print(f"tsrust-deep: {name} round {rnd}: removed users of {sorted(gone)}", flush=True)
                continue
            # prune_round charges an item to the last of its errors: an error that names an
            # item an earlier round removed goes last, so the item counts as that item's
            # cascade, not as a root of a follow-up error in the same item (a missing callee
            # leaves its result's type open: `no field parse on &mut _`).
            located.sort(key=lambda e: any(n in cause_of_name for n in names_in(e[3])))
            before = len(removed)
            unknown = unknown_type_parents(located, copy, cause_of_name)
            try:
                prune.prune_round(located, tool, removed, copy, [], lambda c, m: None)
            except SystemExit as e:
                status[name] = f"stopped: {e}"
                break
            if len(removed) == before:
                status[name] = f"stopped: nothing removed for {errs[0][2]} {errs[0][3][:120]}"
                break
            for r in removed[before:]:
                if tool == "rustc":
                    r["code"] = f"rustc:{r['code']}"
                key = (r["code"], tsrust_full.norm_message(r["message"], copy).removeprefix("unlocated: "))
                # A cascade of an earlier removal: a name the message quotes, a removed impl's
                # method or trait impl, or the removed producer of a value of unknown type.
                parent = next((cause_of_name[n] for n in [*names_in(r["message"]), *impl_names(r["message"])] if n in cause_of_name), None)
                parent = parent or unknown.get((r["file"], r["message"]))
                r["cause"] = list(parent or key)
                r["target"] = name
                for n in [r["name"], *r.get("binds", [])]:
                    cause_of_name.setdefault(n, tuple(r["cause"]))
            (work / "deep.json").write_text(json.dumps({"status": status, "removed": removed}, indent=0))  # partial, each round
            print(f"tsrust-deep: {name} round {rnd}: {len(errs)} errors, {len(removed) - before} items removed", flush=True)
        else:
            status[name] = f"stopped: {a.max_rounds} rounds"
        print(f"tsrust-deep: {name}: {status[name]}", flush=True)


def report(removed, status, copy, work):
    """deep.json and the ranked cause table."""
    causes = {}
    for r in removed:
        c = causes.setdefault(tuple(r["cause"]), {"own": 0, "cascade": 0, "items": 0, "files": set(), "example": f"{r['file']} `{r['name']}`"})
        own = (r["code"], tsrust_full.norm_message(r["message"], copy).removeprefix("unlocated: ")) == tuple(r["cause"])
        c["own" if own else "cascade"] += r["lines"]
        c["items"] += 1
        c["files"].add(r["file"])
    rows = sorted(causes.items(), key=lambda kv: -(kv[1]["own"] + kv[1]["cascade"]))
    (work / "deep.json").write_text(json.dumps({"status": status, "removed": removed}, indent=0))
    print("\n| # | code | message | own lines | cascade lines | items | files | example |\n|---|---|---|---:|---:|---:|---:|---|")
    for i, (k, v) in enumerate(rows, 1):
        print(f"| {i} | {k[0]} | {k[1][:110]} | {v['own']} | {v['cascade']} | {v['items']} | {len(v['files'])} | {v['example']} |")
    print(f"tsrust-deep: {sum(r['lines'] for r in removed)} lines removed in {len(removed)} items, {len(rows)} root causes")


if __name__ == "__main__":
    main()
