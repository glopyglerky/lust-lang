#!/usr/bin/env python3
"""Build a pinned ts-rust slice: the part of a ts-rust crate that lust accepts today.

The slice is ts-rust source, copied verbatim, minus whole items. The loop:
run `lust check` on the slice root, remove the innermost removable item that
holds each error (bench/slices/items lists the items), and repeat until lust
accepts. Then run rustc on the result and prune its errors the same way, then
lust again, until both accept. Every removed item is recorded with the
compiler and the error code that removed it. A rustc-only error means lust
accepted code rustc rejects; the manifest lists those separately (they are
rule-zero bugs or prune side effects, for a human to read). Last, `lust build`
must pass too (the benchmark has build rows); a build failure or panic removes
the fn that causes it, found by the same bisect, and is listed separately.

Removing an item makes its users fail to resolve, so the loop also removes
them: the slice is closed under its own dependencies by construction. It
grows by itself when lust supports more: rerun this script on the same spec.

Usage: prune.py <spec.json> <out-dir> [--lust PATH] [--max-rounds N] [--tsrust DIR [--tsrust-commit SHA]] [--lust-commit SHA]
           [--dev-features a,b | --dev-exclude-features a,b]

--tsrust overrides the spec's tsrust_dir (a copy on a boat sandbox); --tsrust-commit
records the commit when that copy is not a git checkout.

Dev features (docs/dev-features.md): a slice is one synthetic crate, so
cfg(feature = ...) in the copied source needs the package's effective
dev feature list spelled out. Prune reads the project lust config from
the ts-rust tree ([package.metadata.lust] / [workspace.metadata.lust]
`dev-features` or `dev-exclude-features`, or a `.lust.toml` file), or
the --dev-* flags, or $LUST_DEV_FEATURES / $LUST_DEV_EXCLUDE_FEATURES,
and passes the effective list to lust (--features) and to the rustc
oracle (--cfg feature="..."). `prune.json` records it as
`dev_features` with its source. With no config anywhere the old slice
behavior stands (no features) and the manifest says so. For ts-rust,
dev builds run with jemalloc off: `--dev-exclude-features jemalloc`
on a tree that has no lust config yet.

spec.json (see bench/slices/*/spec.json):
  tsrust_dir   ts-rust checkout (read only; default /home/theo/.t3/projects/ts-rust)
  src_dir      dir in ts-rust that holds the crate's files (copied whole)
  root_from    the crate root file in src_dir; copied to `root` with `append` added
  root         new root file name, written next to root_from (the bin crate entry)
  append       text appended to the root (stand-in mods, fn main)
  support      extra files copied into src_dir of the slice: {dest: path-in-lust-repo}
  rewrite      extern crate names whose paths become `crate::<name>::` (stand-ins)
  rewrite_to   {name: prefix}: `name::` becomes `prefix::` (a part crate merged into this one)
  root_replace optional [[old, new], ...] text edits of the root (the root is bench text,
               not a ts-rust module file; use it to unmount a module lust cannot take)
  unmount      optional [name, ...]: comment out the root's `pub mod <name>;` lines (a smaller
               slice of the same crate; users of an unmounted module are pruned)
  drop         optional [[file, item name, reason], ...]: items removed before the first round,
               for a lust failure the loop cannot locate (a build error outside fn bodies).
               Recorded in prune.json with "by": "spec"; their users are pruned as usual.

Only the files in root_from's module tree are copied.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ITEMS = HERE / "items/target/release/slice-items"


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def build_items_tool():
    if not ITEMS.exists():
        r = sh(["cargo", "build", "--release", "-q", "--offline"], cwd=HERE / "items")
        if r.returncode != 0:
            sys.exit(f"slice-items build failed:\n{r.stderr[-2000:]}")


def tsrust_dir(spec):
    return Path(spec.get("tsrust_dir", "/home/theo/.t3/projects/ts-rust"))


def _toml(path):
    import tomllib

    try:
        return tomllib.loads(Path(path).read_text())
    except (OSError, ValueError):
        return {}


def _expand_features(feature_table, selected):
    """Plain-name expansion through a [features] table (dep: and
    pkg/feat entries enable no cfg feature of this package)."""
    out, queue = [], list(selected)
    while queue:
        name = queue.pop()
        if ":" in name or "/" in name or name in out:
            continue
        out.append(name)
        queue += [v for v in feature_table.get(name, []) if ":" not in v and "/" not in v]
    return sorted(out)


def dev_features_for(spec, cli_features=None, cli_exclude=None, env=None):
    """(effective feature list, source note) for the slice's package.

    Order: CLI flags, env, the ts-rust tree's lust config (package
    manifest, workspace manifest, then .lust.toml up from src_dir),
    then the spec's own `features` list, then no features.
    """
    env = os.environ if env is None else env
    split = lambda v: [s for s in re.split(r"[,\s]+", v or "") if s]  # noqa: E731
    exact = split(cli_features) or split(env.get("LUST_DEV_FEATURES", ""))
    exclude = split(cli_exclude) or split(env.get("LUST_DEV_EXCLUDE_FEATURES", ""))
    if exact and exclude:
        sys.exit("dev features: give only one of exact list and exclude list")
    src = tsrust_dir(spec) / spec["src_dir"]
    # The package manifest above src_dir owns the [features] table.
    pkg, pkg_path = {}, None
    d = src
    while d != d.parent:
        cand = d / "Cargo.toml"
        data = _toml(cand) if cand.exists() else {}
        if "package" in data:
            pkg, pkg_path = data, cand
            break
        d = d.parent
    table = pkg.get("features", {})
    defaults = [f for f in table.get("default", []) if ":" not in f and "/" not in f]
    if exact:
        return _expand_features(table, exact), "flag or env (--dev-features)"
    if exclude:
        return _expand_features(table, [f for f in defaults if f not in exclude]), "flag or env (--dev-exclude-features)"
    cfg = {}
    where = ""
    lust = pkg.get("package", {}).get("metadata", {}).get("lust", {})
    if lust:
        cfg, where = lust, f"[package.metadata.lust] in {pkg_path}"
    if not cfg:
        d = src
        while d != d.parent and not cfg:
            data = _toml(d / "Cargo.toml") if (d / "Cargo.toml").exists() else {}
            lust = data.get("workspace", {}).get("metadata", {}).get("lust", {})
            if lust:
                cfg, where = lust, f"[workspace.metadata.lust] in {d / 'Cargo.toml'}"
            d = d.parent
    if not cfg:
        d = src
        while d != d.parent and not cfg:
            data = _toml(d / ".lust.toml") if (d / ".lust.toml").exists() else {}
            if "dev-features" in data or "dev-exclude-features" in data:
                cfg, where = data, str(d / ".lust.toml")
            d = d.parent
    if "dev-features" in cfg:
        return _expand_features(table, cfg["dev-features"]), where
    if "dev-exclude-features" in cfg:
        ex = cfg["dev-exclude-features"]
        return _expand_features(table, [f for f in defaults if f not in ex]), where
    if spec.get("features"):
        return _expand_features(table, spec["features"]), "spec features"
    return [], "no lust dev config found; no features (old slice behavior)"


# Effective dev features of the current prune run, set in main().
DEV_FEATURES = []


def lust_feature_args():
    return ["--features", ",".join(DEV_FEATURES)] if DEV_FEATURES else []


def rustc_feature_args():
    out = []
    for f in DEV_FEATURES:
        out += ["--cfg", f'feature="{f}"']
    return out


def copy_tree(spec, out):
    """Copy the crate's module tree: write the new root next to root_from, walk the tree
    from it (so `append` can mount more modules), and keep only the files it reaches."""
    src = tsrust_dir(spec) / spec["src_dir"]
    dst = out / "src"
    for p in src.rglob("*.rs"):
        (dst / p.relative_to(src)).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(p, dst / p.relative_to(src))
    root_text = (src / spec["root_from"]).read_text()
    for old, new in spec.get("root_replace", []):
        if old not in root_text:
            sys.exit(f"root_replace: text not found: {old!r}")
        root_text = root_text.replace(old, new, 1)
    for name in spec.get("unmount", []):
        pat = re.compile(rf"^((///|#\[)[^\n]*\n)*(pub(\([^)]*\))? )?mod {re.escape(name)};\n", re.M)
        if len(pat.findall(root_text)) != 1:
            sys.exit(f"unmount: `mod {name};` must occur once in the root")
        root_text = pat.sub(f"// bench slice: `{name}` not mounted.\n", root_text)
    (dst / spec["root"]).write_text(root_text + spec.get("append", ""))
    for d, s in spec.get("support", {}).items():
        (dst / d).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(ROOT / s, dst / d)
    r = sh([str(ITEMS), "--tree", str(dst / spec["root"])])
    if r.returncode != 0:
        sys.exit(f"slice-items --tree: {r.stderr.strip()}")
    keep = {str(Path(l).resolve()) for l in r.stdout.splitlines()}
    for p in list(dst.rglob("*.rs")):
        if str(p.resolve()) not in keep:
            p.unlink()
    for d in sorted((x for x in dst.rglob("*") if x.is_dir()), reverse=True):
        if not any(d.iterdir()):
            d.rmdir()
    names = spec.get("rewrite", [])
    if names:
        pat = re.compile(r"(?<![\w:])(" + "|".join(map(re.escape, names)) + r")::")
        for p in dst.rglob("*.rs"):
            if str(p.relative_to(dst)) in spec.get("support", {}):
                continue
            t = p.read_text()
            n = pat.sub(r"crate::\1::", t)
            if n != t:
                p.write_text(n)
    # rewrite_to: whole-path renames, such as a part crate merged into this one
    # (`goport_util::x` becomes `crate::x`).
    for name, to in spec.get("rewrite_to", {}).items():
        pat = re.compile(rf"(?<![\w:]){re.escape(name)}::")
        for p in dst.rglob("*.rs"):
            t = p.read_text()
            n = pat.sub(to + "::", t)
            if n != t:
                p.write_text(n)
    return dst / spec["root"]


def lust_errors(lust, root, env, strict=True, verb="check"):
    """Located lust errors of `lust <verb>`. A build that fails with no located error
    (a panic) gives one error at line 0, which the caller bisects."""
    cmd = [lust, verb, str(root), "--json", "--max-errors", "100000"] + lust_feature_args()
    if verb == "build":
        cmd += ["-o", str(Path(env["LUST_CACHE_DIR"]) / "slice-bin")]
    r = sh(cmd, env=env)
    errs, other = [], []
    for line in r.stdout.splitlines() + r.stderr.splitlines():
        if not line.startswith("{"):
            continue
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("severity") != "error":
            continue
        sp = d.get("span") or {}
        if d.get("file") and sp.get("line"):
            line = sp["line"] if located(d["file"], sp) else 0
            errs.append((d["file"], line, d.get("code", ""), d.get("message", "")))
        else:
            other.append(d.get("message", ""))
    # A failed build with no located error, or a panic of either verb: one error at line 0,
    # which the caller bisects. Numbers in parens (ids) are dropped so probes compare equal.
    if r.returncode != 0 and not errs and (verb == "build" or "panicked at" in r.stderr):
        lines = [l for l in r.stderr.splitlines() if l.strip()]
        first = re.sub(r"\(\d+\)", "", lines[0] if lines else "")
        # Every tcc failure has the same first line; add the C error with its file, line, and
        # generated type numbers dropped, so the bisect tracks this failure and not another one.
        cerr = next((l for l in lines[1:] if ": error: " in l), "")
        if cerr:
            first += " | " + re.sub(r"\d+", "N", cerr.split(": error: ", 1)[1])
        return [(str(root), 0, "BUILD" if verb == "build" else "PANIC", first[:300])]
    if strict and r.returncode != 0 and not errs:
        tail = (r.stderr or r.stdout)[-1500:]
        sys.exit(f"lust failed with no located error (exit {r.returncode}):\n{other}\n{tail}")
    return errs


def located(path, sp):
    """True when the span's byte offset is at its line and column in the file on disk.

    lust reports some spans in its own std source under the user's root file name;
    those fail this check and go to the bisect instead.
    """
    try:
        data = Path(path).read_bytes()
    except OSError:
        return False
    off = sp.get("start", -1)
    if not 0 <= off <= len(data):
        return False
    line = data.count(b"\n", 0, off) + 1
    col = len(data[data.rfind(b"\n", 0, off) + 1:off].decode("utf-8", "replace")) + 1
    return line == sp["line"] and col == sp.get("col", col)


def rustc_errors(root, work):
    r = sh(["rustc", "--edition", "2024", "--crate-type", "bin", "--emit=metadata", "-A", "warnings"]
           + rustc_feature_args() + ["--error-format=short", "-o", str(work / "slice.rmeta"), str(root)])
    errs = []
    for line in r.stderr.splitlines():
        m = re.match(r"^(.+?\.rs):(\d+):(\d+): error(\[E\d+\])?: (.*)$", line)
        if m:
            f = m.group(1)
            if not os.path.isabs(f):
                f = str((Path.cwd() / f).resolve())
            errs.append((f, int(m.group(2)), (m.group(4) or "").strip("[]"), m.group(5)))
    if r.returncode != 0 and not errs:
        sys.exit(f"rustc failed with no located error:\n{r.stderr[-1500:]}")
    return errs


def items_of(path, cache):
    if path not in cache:
        r = sh([str(ITEMS), path])
        if r.returncode != 0:
            sys.exit(f"slice-items: {r.stderr.strip()}")
        cache[path] = [(int(a), int(b), int(c), k, n) for a, b, c, k, n in (l.split() for l in r.stdout.splitlines())]
    return cache[path]


def apply_drops(spec, src_root, removed):
    """Remove the spec's `drop` items (one item of that name in the file) before the loop."""
    for f, name, why in spec.get("drop", []):
        p = src_root / f
        hits = [it for it in items_of(str(p), {}) if it[4] == name]
        if len(hits) != 1:
            sys.exit(f"drop: {len(hits)} items named {name} in {f}")
        it = hits[0]
        lines = p.read_text().split("\n")
        del lines[it[0] - 1:it[1]]
        p.write_text("\n".join(lines))
        removed.append({"file": f, "kind": it[3], "name": name, "lines": it[1] - it[0] + 1, "by": "spec",
                        "code": "DROP", "message": why[:160]})


def short_type(t):
    """`a::B<C, D<E>>` -> `B`: a type or trait as an impl key names it (generics, paths, `&`,
    `dyn` and lifetimes dropped)."""
    t = re.sub(r"\s+", " ", t)
    while re.search(r"<[^<>]*>", t):
        t = re.sub(r"<[^<>]*>", "", t)
    t = re.sub(r"^(&\s*('\w+\s+)?(mut\s+)?|dyn\s+)+", "", t.strip())
    return t.split("::")[-1].strip()


def impl_keys(text, fns):
    """Keys a removed impl block leaves dangling, for the deep census's cascade lookup
    (tsrust_deep.impl_names): `Type::method` per fn in it, `impl Trait for Type` for a trait impl."""
    head = re.sub(r"\s+", " ", text.split("{", 1)[0])
    m = re.search(r"\bimpl\b(.*)$", head)
    if not m:
        return []
    body = m.group(1)
    while re.search(r"<[^<>]*>", body):
        body = re.sub(r"<[^<>]*>", "", body)
    body = re.split(r"\bwhere\b", body)[0].strip().lstrip("!")
    trait, _, ty = body.rpartition(" for ")
    ty = short_type(ty or body)
    return [f"{ty}::{f}" for f in fns] + ([f"impl {short_type(trait)} for {ty}"] if trait else [])


def prune_round(errs, tool, removed, src_root, unlocated, relocate):
    """Remove the innermost item holding each error. Returns the number of items removed."""
    by_file = {}
    cache = {}
    n_leaf = 0

    def innermost(f, line):
        hits = [it for it in items_of(f, cache) if it[0] <= line <= it[1]]
        # The deepest; on a tie (an impl and its fn share a line) the shortest.
        return max(hits, key=lambda t: (t[2], -(t[1] - t[0]))) if hits else None

    for f, line, code, msg in errs:
        f = str(Path(f).resolve())
        if not f.startswith(str(src_root)):
            sys.exit(f"error outside the slice ({f}:{line}): {code} {msg}")
        it = innermost(f, line)
        if it and it[3] == "use" and (bound := drop_use_leaf(f, it, code, msg)):
            removed.append({"file": os.path.relpath(f, src_root), "kind": "use-leaf", "name": bound,
                            "lines": 0, "by": tool, "code": code, "message": msg[:160]})
            n_leaf += 1
            continue
        if it:
            by_file.setdefault(f, {})[(it[0], it[1])] = (it, code, msg)
            continue
        # No usable location: lust put the span on line 1, or on one of its std lines
        # that it reports under the root file. Find the fn by bisecting body stubs.
        hit = relocate(code, msg)
        if hit:
            p, fn_line = hit
            it = innermost(p, fn_line)
            by_file.setdefault(p, {})[(it[0], it[1])] = (it, code, "unlocated: " + msg)
            unlocated.append({"code": code, "message": msg[:160], "found_in": f"{os.path.relpath(p, src_root)}:{fn_line}"})
            continue
        # Not in a body (a bound or an impl header). When the message names one item,
        # as in "unknown trait `TryFrom`", remove the items whose text names it.
        names = re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)`", msg)
        if len(names) != 1:
            sys.exit(f"cannot locate the error reported at {f}:{line}: {code} {msg}")
        found = []
        for p in sorted(src_root.rglob("*.rs")):
            for i, l in enumerate(p.read_text().split("\n"), 1):
                if re.search(rf"\b{names[0]}\b", l) and (it := innermost(str(p), i)):
                    by_file.setdefault(str(p), {})[(it[0], it[1])] = (it, code, "unlocated: " + msg)
                    found.append(f"{os.path.relpath(p, src_root)}:{i}")
        if not found:
            sys.exit(f"cannot locate the error reported at {f}:{line}: {code} {msg}")
        unlocated.append({"code": code, "message": msg[:160], "found_in": found[:20]})
    n = 0
    for f, spans in by_file.items():
        lines = Path(f).read_text().split("\n")
        # Drop nested duplicates: keep outer spans only when an outer one is also listed.
        keep = sorted(spans)
        outer = []
        for s in keep:
            if outer and s[0] >= outer[-1][0] and s[1] <= outer[-1][1]:
                continue
            outer.append(s)
        for s in sorted(outer, reverse=True):
            it, code, msg = spans[s]
            # A whole `use` names no item, but its renames (`a as b`) are names users refer to.
            binds = re.findall(r"\bas\s+([A-Za-z]\w*)", "\n".join(lines[s[0] - 1:s[1]])) if it[3] == "use" else []
            if it[3] in ("impl", "traitimpl"):
                fns = [x[4] for x in items_of(f, cache) if x[3] == "fn" and s[0] <= x[0] and x[1] <= s[1]]
                binds = impl_keys("\n".join(lines[s[0] - 1:s[1]]), fns)
            del lines[s[0] - 1:s[1]]
            removed.append({"file": os.path.relpath(f, src_root), "kind": it[3], "name": it[4],
                            "lines": s[1] - s[0] + 1, "by": tool, "code": code, "message": msg[:160]}
                           | ({"binds": binds} if binds else {}))
            n += 1
        Path(f).write_text("\n".join(lines))
    return n + n_leaf


def drop_use_leaf(f, it, code, msg):
    """For an unresolved or private import inside a `use a::{b, c}` group, delete only the
    failing name, in place (line count unchanged), so the other names stay imported. Returns
    the name the leaf bound (its rename for `b as x`), or False (remove the whole item) for a
    plain `use a::b;` or an ambiguous match."""
    if code not in ("L0506", "E0432", "L0510", "E0603") or msg.count("`") < 2:
        return False
    leaf = msg.split("`")[1].split("::")[-1]
    lines = Path(f).read_text().split("\n")
    text = "\n".join(lines[it[0] - 1:it[1]])
    if "{" not in text or not re.fullmatch(r"[A-Za-z_]\w*", leaf):
        return False
    pat = re.compile(rf"(?<![\w:]){leaf}(?!\w)(\s+as\s+\w+)?(::\*|::\{{[^{{}}]*\}})?\s*,?[ \t]*")
    hits = [(i, m) for i in range(it[0] - 1, it[1]) for m in pat.finditer(lines[i])]
    if len(hits) != 1:
        return False
    i, m = hits[0]
    lines[i] = lines[i][:m.start()] + lines[i][m.end():]
    Path(f).write_text("\n".join(lines))
    alias = m.group(1).split()[-1] if m.group(1) else "_"
    return leaf if alias == "_" else alias


def bisect(lust, root, env, src_root, code, msg, verb="check"):
    """Find the fn whose body makes lust report (code, msg) with no usable span.

    Stubs fn bodies as `{ loop {} }` in halves: a body set that makes the error go
    away when stubbed holds the culprit. Files are restored after every probe.
    Returns (file, first line of the fn) or None when no body explains the error.
    """
    bodies = []
    for p in sorted(src_root.rglob("*.rs")):
        r = sh([str(ITEMS), "--bodies", str(p)])
        rows = sorted(tuple(map(int, l.split())) for l in r.stdout.splitlines())
        outer = []
        for b in rows:
            if outer and (b[0], b[1]) <= (outer[-1][2], outer[-1][3]):
                continue  # nested in the previous body
            outer.append(b)
        bodies += [(str(p), b) for b in outer]
    originals = {}

    def has_error(stub):
        by = {}
        for p, b in stub:
            by.setdefault(p, []).append(b)
        for p, bs in by.items():
            text = originals.setdefault(p, Path(p).read_text())
            lines = text.split("\n")
            for ol, oc, cl, cc, _ in sorted(bs, reverse=True):
                head = lines[ol - 1][:oc - 1]
                tail = lines[cl - 1][cc:]
                lines[ol - 1:cl] = [head + "{ loop {} }" + tail]
            Path(p).write_text("\n".join(lines))
        try:
            return any(c == code and m == msg for _, _, c, m in lust_errors(lust, root, env, strict=False, verb=verb))
        finally:
            for p in by:
                Path(p).write_text(originals[p])

    if has_error(bodies):
        return None
    zero, cand = [], bodies
    while len(cand) > 1:
        half, rest = cand[:len(cand) // 2], cand[len(cand) // 2:]
        if not has_error(zero + half):
            cand = half
        else:
            zero, cand = zero + half, rest
    p, b = cand[0]
    return p, b[4]


def write_manifest(path, m):
    """prune.json: the summary fields indented, then one removed item per line."""
    by_code = {}
    for r in m["removed"]:
        k = r["by"] + " " + r["code"]
        by_code[k] = by_code.get(k, 0) + r["lines"]
    m = dict(m, removed_lines_by_oracle_and_code=dict(sorted(by_code.items(), key=lambda kv: -kv[1])))
    head = {k: v for k, v in m.items() if k != "removed"}
    text = json.dumps(head, indent=1)[:-2] + ',\n "removed": [\n'
    text += ",\n".join("  " + json.dumps(r) for r in m["removed"]) + "\n ]\n}\n"
    path.write_text(text)


def count_lines_file(p):
    return sum(1 for _ in open(p))


def nonblank_lines(src):
    return sum(sum(1 for l in open(p) if l.strip() and not l.strip().startswith("//")) for p in src.rglob("*.rs"))


def count_lines(src):
    return sum(sum(1 for _ in open(p)) for p in src.rglob("*.rs"))


def lines_by_module(src):
    """Lines per top-level module (first path part, `.rs` dropped) of the slice's src."""
    out = {}
    for p in src.rglob("*.rs"):
        k = p.relative_to(src).parts[0].removesuffix(".rs")
        out[k] = out.get(k, 0) + count_lines_file(p)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("spec")
    ap.add_argument("out")
    ap.add_argument("--lust", default=str(ROOT / "target/release/lust"))
    ap.add_argument("--max-rounds", type=int, default=400)
    ap.add_argument("--tsrust", help="ts-rust checkout to copy from (overrides spec tsrust_dir)")
    ap.add_argument("--tsrust-commit", help="ts-rust commit to record when --tsrust is not a git checkout")
    ap.add_argument("--lust-commit", help="lust commit to record when this tree is not a git checkout (boat)")
    ap.add_argument("--dev-features", help="exact dev feature list (comma separated); overrides the tree config")
    ap.add_argument("--dev-exclude-features", help="features removed from the package default list (comma separated)")
    a = ap.parse_args()
    if a.dev_features and a.dev_exclude_features:
        ap.error("give only one of --dev-features and --dev-exclude-features")
    spec = json.loads(Path(a.spec).read_text())
    if a.tsrust:
        spec["tsrust_dir"] = a.tsrust
    global DEV_FEATURES
    DEV_FEATURES, dev_source = dev_features_for(spec, a.dev_features, a.dev_exclude_features)
    print(f"dev features: {DEV_FEATURES or '[]'} ({dev_source})", flush=True)
    out = Path(a.out).resolve()
    if out.exists():
        shutil.rmtree(out)
    build_items_tool()
    root = copy_tree(spec, out)
    src_root = (out / "src").resolve()
    start_lines = count_lines(src_root)
    start_by_module = lines_by_module(src_root)
    t0 = time.time()
    env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(out / ".lust-cache"))
    removed = []
    apply_drops(spec, src_root, removed)
    unlocated = []
    rounds = 0
    # Three oracles in turn until all accept: lust check, rustc, lust build.
    tools = ["lust", "rustc", "lust-build"]
    tool = "lust"
    clean = set()
    while rounds < a.max_rounds:
        rounds += 1
        if tool == "rustc":
            errs = rustc_errors(root, out)
        else:
            errs = lust_errors(a.lust, root, env, verb="build" if tool == "lust-build" else "check")
        if not errs:
            clean.add(tool)
            if len(clean) == len(tools):
                break
            tool = tools[(tools.index(tool) + 1) % len(tools)]
            continue
        clean = set()
        verb = "build" if tool == "lust-build" else "check"
        n = prune_round(errs, tool, removed, src_root, unlocated,
                        lambda c, m: bisect(a.lust, root, env, src_root, c, m, verb))
        print(f"round {rounds} ({tool}): {len(errs)} errors, removed {n} items, {count_lines(src_root)} lines left, "
              f"{time.time() - t0:.0f} s", flush=True)
        # The removals so far, for coverage.py when a later round stops on an error it cannot locate.
        write_manifest(out / "prune-partial.json", {"rounds": rounds, "removed": removed})
    else:
        sys.exit("prune did not converge")
    # Keep only the files rustc loads on this target: cfg-disabled modules (windows,
    # darwin) and unmounted files are not part of the crate. Both compilers must
    # still accept after that.
    dep = out / "slice.d"
    sh(["rustc", "--edition", "2024", "--crate-type", "bin", "--emit=dep-info", "-A", "warnings"]
       + rustc_feature_args() + ["-o", str(dep), str(root)])
    loaded = {str(Path(t).resolve()) for t in re.findall(r"(\S+\.rs):", dep.read_text())}
    dropped = 0
    for p in list(src_root.rglob("*.rs")):
        if str(p.resolve()) not in loaded:
            dropped += count_lines_file(p)
            p.unlink()
    for d in sorted((x for x in src_root.rglob("*") if x.is_dir()), reverse=True):
        if not any(d.iterdir()):
            d.rmdir()
    if lust_errors(a.lust, root, env) or rustc_errors(root, out) or lust_errors(a.lust, root, env, verb="build"):
        sys.exit("the slice no longer checks after dropping unloaded files")
    dep.unlink()
    shutil.rmtree(out / ".lust-cache", ignore_errors=True)
    (out / "slice.rmeta").unlink(missing_ok=True)
    commit = a.tsrust_commit or sh(["git", "-C", str(tsrust_dir(spec)), "rev-parse", "HEAD"]).stdout.strip()
    spec.pop("tsrust_dir", None) if a.tsrust else None
    end_by_module = lines_by_module(src_root)
    lust_commit = a.lust_commit or sh(["git", "-C", str(ROOT), "rev-parse", "HEAD"]).stdout.strip()
    manifest = {
        "tsrust_commit": commit, "lust_commit": lust_commit, "spec": spec,
        "dev_features": DEV_FEATURES, "dev_features_source": dev_source,
        "lines_before": start_lines, "lines": count_lines(src_root),
        "code_lines": nonblank_lines(src_root), "unloaded_lines_dropped": dropped, "rounds": rounds,
        "prune_seconds": round(time.time() - t0),
        "lines_by_module": {k: [v, end_by_module.get(k, 0)] for k, v in sorted(start_by_module.items())},
        "removed_items": len(removed),
        "removed_by_rustc": [r for r in removed if r["by"] == "rustc"],
        "removed_by_lust_build": [r for r in removed if r["by"] == "lust-build"],
        "unlocated_errors": unlocated,
        "removed": removed,
    }
    write_manifest(out / "prune.json", manifest)
    (out / "prune-partial.json").unlink(missing_ok=True)
    print(f"slice: {manifest['lines']} lines ({manifest['code_lines']} code) of {start_lines} kept, {len(removed)} items removed "
          f"({len(manifest['removed_by_rustc'])} by rustc), {rounds} rounds")


if __name__ == "__main__":
    main()
