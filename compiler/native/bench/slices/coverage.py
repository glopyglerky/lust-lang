#!/usr/bin/env python3
"""Real ts-rust coverage from pruned slices: docs/real-coverage.md.

A slice (bench/slices/<name>, made by prune.py) holds the ts-rust code that
lust check, rustc, and lust build all accept. This script counts, per ts-rust
crate and per top module, how many lines of the original files are still in
the slice, and ranks the reasons prune.py removed code.

Root causes: removing an item makes its users fail ("cannot find type `X`").
A removal whose message names an item that was removed earlier (or a name
that is not in the slice at all, such as an unstubbed external crate) is a
cascade. Its lines are charged to the first removal in the chain, so the
ranked list shows what to fix in lust, not the noise after it.

Usage: coverage.py --out docs/real-coverage.md <slice-dir> [<slice-dir> ...]
  The first slice dir is the main one (the widest; per-crate tables use it).
  --gaps docs/ts-rust-gaps.json gives the universe (active prod and generated
  files on this target) and port-scan-sync.py's estimate to compare with.
"""
import argparse
import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ITEMS = HERE / "items/target/release/slice-items"
SRC = "crates/ts_goport/src"
NOTES_MARK = "<!-- hand notes: kept by coverage.py -->"
BENCH_FILES = {"slice_main.rs", "support.rs", "support_smallvec.rs", "support_xxhash.rs"}


def crate_of_files(tsrust):
    """{path under SRC: crate} for the two part crates; everything else is ts_goport."""
    out = {}
    src = tsrust / SRC
    for crate, root in [("goport_util", "goport_util_root.rs"), ("goport_lsproto", "goport_lsproto_root.rs")]:
        r = subprocess.run([str(ITEMS), "--tree", str(src / root)], capture_output=True, text=True, check=True)
        for line in r.stdout.splitlines():
            f = Path(line).resolve()
            if f.is_relative_to(src.resolve()):
                out.setdefault(str(f.relative_to(src.resolve())), crate)
    return out


def test_lines(text):
    """Lines of inline `#[cfg(test)] mod x { .. }` blocks (neither compiler loads them)."""
    lines = text.split("\n")
    n, i = 0, 0
    while i < len(lines):
        if lines[i].strip() == "#[cfg(test)]" and i + 1 < len(lines) and re.match(r"\s*(pub )?mod \w+ \{", lines[i + 1]):
            depth, j = 0, i + 1
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                if depth <= 0:
                    break
                j += 1
            n += j - i + 1
            i = j + 1
            continue
        i += 1
    return n


def file_lines(p):
    t = p.read_text()
    return t.count("\n") + (0 if t.endswith("\n") else 1), test_lines(t)


def top_module(rel):
    return rel.split("/")[0].removesuffix(".rs")


# Crates ts-rust uses that the slices do not stub (support.rs stubs rustc_hash, indexmap, memchr).
EXTERNAL = {"smallvec", "SmallVec", "xxhash_rust", "bumpalo", "flate2", "jiff", "rustix", "signal_hook", "nix", "num_bigint",
            "num_traits", "ryu_js", "tikv_jemallocator", "tikv_jemalloc_ctl", "miow", "same_file", "name_to_handle_at",
            "notify", "unicode_normalization", "lzma_rust2"}
DERIVE = re.compile(r"^deriving `\w+` requires every field|^cannot derive")
# Messages that say a name does not resolve: the only ones that can be a cascade.
UNRESOLVED = re.compile(r"cannot find|unresolved|not found|no method|no field|no associated|no variant|unknown|does not exist|failed to resolve")


def root_causes(removed, item_text=None):
    """Charge each removal's lines to its root cause. Returns the roots (removals that are no
    cascade), each with own_lines, cascade_lines, cascade_items. See the module comment.

    lust names the module, not the item, when a path's last segment is missing (`ata::Gone`
    gives "cannot find type `ata`"), and a missing item behind a longer path gives L0505
    "module paths in types is not supported yet". For those, `item_text` (removal -> its
    ts-rust source text) finds the `name::Item` (or any `::Item`) paths in the removed item,
    and the first Item that prune.py removed is the parent.

    A removal is a cascade when its message says a name does not resolve and names an item that
    prune.py removed (in any round order, as one round removes users and the item together), or a
    name that an earlier removal already named (a std item lust lacks, an external crate, an
    unmounted module): the first removal that names it is the root.
    """
    item_at = {}
    for i, r in enumerate(removed):
        if r["name"] != "-":
            item_at.setdefault(r["name"], i)
    parent = list(range(len(removed)))
    # A removed `use` (an import lust rejects) is the root of every later "cannot find" that
    # names what it imported (`with_data!` after its `pub(crate) use with_data;` went).
    named_first = {}
    for i, r in enumerate(removed):
        if r["kind"] == "use":
            for nm in re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)`", r["message"]):
                named_first.setdefault(nm, i)
    for i, r in enumerate(removed):
        msg = r["message"]
        # A derive that fails on a field whose type prune.py removed earlier is that
        # type's cascade (lust reports the derive before "cannot find type").
        if item_text and DERIVE.search(msg):
            j = next((item_at[x] for x in re.findall(r"\b([A-Z]\w*)\b", item_text(r)) if item_at.get(x, i) < i), None)
            if j is not None:
                parent[i] = j
            continue
        if item_text and msg.startswith("module paths in"):
            j = next((item_at[x] for x in re.findall(r"::([A-Z]\w*)", item_text(r)) if item_at.get(x, i) != i), None)
            if j is not None:
                parent[i] = j
            continue
        if not UNRESOLVED.search(msg):
            continue
        for nm in re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)", msg):
            j = item_at.get(nm)
            if j is None and item_text and nm[:1].islower():
                j = next((item_at[x] for x in re.findall(rf"\b{nm}::(?:[a-z_]\w*::)*([A-Z]\w*)", item_text(r))
                          if item_at.get(x, i) != i), None)
            if j is not None and j != i and removed[j]["name"] != r["name"]:
                parent[i] = j
                break
            if j is None:
                if nm in named_first:
                    parent[i] = named_first[nm]
                else:
                    named_first[nm] = i
                break

    def root(i):
        seen = set()
        while parent[i] != i and i not in seen:
            seen.add(i)
            i = parent[i]
        return min(seen | {i}) if parent[i] != i else i

    roots = {}
    for i, r in enumerate(removed):
        k = root(i)
        if k not in roots:
            rr = removed[k]
            ext = next((nm for nm in re.findall(r"`([A-Za-z_][A-Za-z0-9_]*)", rr["message"]) if nm in EXTERNAL), None)
            roots[k] = dict(rr, own_lines=0, cascade_lines=0, cascade_items=0,
                            **({"by": "-", "code": "external", "message": f"`{ext}` (external crate, no stand-in)"} if ext else {}))
        if k == i:
            roots[k]["own_lines"] += r["lines"]
        else:
            roots[k]["cascade_lines"] += r["lines"]
            roots[k]["cascade_items"] += 1
    return list(roots.values())


def item_source(path, kind, name, cache={}):
    """Source text of the item `kind name` in a ts-rust file (the whole file for an unnamed
    item, or when slice-items does not list it); "" when the file is gone."""
    if not path.exists():
        return ""
    if path not in cache:
        r = subprocess.run([str(ITEMS), str(path)], capture_output=True, text=True, check=True)
        cache[path] = ([l.split() for l in r.stdout.splitlines()], path.read_text().split("\n"))
    items, lines = cache[path]
    span = next(((int(a), int(b)) for a, b, _, k, n in items if k == kind and n == name), None)
    return "\n".join(lines[span[0] - 1:span[1]] if span else lines)


def reason_key(r):
    """Group key of a root cause: oracle, code, message with the item names kept only when short."""
    msg = re.sub(r"^unlocated: ", "", r["message"])
    msg = re.sub(r"\b\d+\b", "N", msg)
    return (r["by"], r["code"], msg[:90])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("slices", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--gaps", default=str(ROOT / "docs/ts-rust-gaps.json"))
    ap.add_argument("--tsrust", default="/home/theo/.t3/projects/ts-rust")
    ap.add_argument("--top", type=int, default=30)
    a = ap.parse_args()
    tsrust = Path(a.tsrust)
    gaps = json.loads(Path(a.gaps).read_text())
    universe = {f["file"].removeprefix(SRC + "/"): f for f in gaps["files"]
                if f["file"].startswith(SRC + "/") and f["scope"] == "active" and f["cat"] in ("prod", "gen")
                and not f["file"].startswith(SRC + "/bin/")}
    crates = crate_of_files(tsrust)

    md = ["# Real ts-rust coverage", "",
          "Generated by `python3 bench/slices/coverage.py` from the pruned slices. Do not edit the numbers by hand.", ""]
    main_dir = Path(a.slices[0])
    summary_rows = []
    for d in map(Path, a.slices):
        m = json.loads((d / "prune.json").read_text())
        src = d / "src"
        kept = {str(p.relative_to(src)): p for p in src.rglob("*.rs") if str(p.relative_to(src)) not in BENCH_FILES}
        acc = sum(file_lines(p)[0] - file_lines(p)[1] for p in kept.values())
        summary_rows.append((d.name, m["lines_before"], m["lines"], acc, m["rounds"], m["removed_items"], m["lust_commit"][:9]))
        if d != main_dir:
            continue
        main_m, main_kept = m, kept

    md += ["## Slices", "",
           "| Slice | lines copied | lines kept | ts-rust code lines kept (no bench files, no cfg(test) blocks) | prune rounds | items removed | lust |",
           "|---|---:|---:|---:|---:|---:|---|"]
    md += [f"| `{n}` | {b:,} | {k:,} | {c:,} | {r} | {i:,} | `{lc}` |" for n, b, k, c, r, i, lc in summary_rows]
    md += ["", f"ts-rust `{main_m['tsrust_commit'][:9]}`. The per-crate tables below use `{main_dir.name}`.", ""]

    # Per crate and per top module: universe lines (active prod + generated, cfg(test) blocks out)
    # vs lines kept in the slice, with the port-scan estimate next to it.
    per = defaultdict(lambda: [0, 0, 0, 0])  # universe, kept, estimate, generated
    for rel, f in universe.items():
        p = tsrust / SRC / rel
        total, tst = file_lines(p)
        key = (crates.get(rel, "ts_goport"), top_module(rel))
        per[key][0] += total - tst
        if not f["missing_features"] and not f["generated"]:
            per[key][2] += total - tst
        if f["generated"]:
            per[key][3] += total - tst
        if rel in main_kept:
            k, kt = file_lines(main_kept[rel])
            per[key][1] += k - kt
    md += ["## Lines lust really accepts, per ts-rust crate", "",
           "Universe: the crate's active files on x86_64 Linux (prod and generated, `src/bin` out), minus inline",
           "`#[cfg(test)]` blocks. Accepted: those lines still in the slice, so lust check, rustc, and lust build",
           "all accept them. Program output equal to rustc is checked only for the code the bench driver runs",
           "(tspath fns; docs/speed.md). Estimate: port-scan-sync.py's files with no missing feature (generated",
           "files are outside its universe, so they count 0 there).", "",
           "| crate | universe lines | accepted | % | port-scan estimate | % |", "|---|---:|---:|---:|---:|---:|"]
    by_crate = defaultdict(lambda: [0, 0, 0])
    for (c, _), v in per.items():
        for i in range(3):
            by_crate[c][i] += v[i]
    tot = [sum(v[i] for v in by_crate.values()) for i in range(3)]
    pc = lambda x, y: f"{100 * x / y:.1f}" if y else "-"
    for c in ["goport_util", "goport_lsproto", "ts_goport"]:
        u, k, e = by_crate[c]
        md.append(f"| {c} | {u:,} | {k:,} | {pc(k, u)} | {e:,} | {pc(e, u)} |")
    md.append(f"| **all** | {tot[0]:,} | **{tot[1]:,}** | **{pc(tot[1], tot[0])}** | {tot[2]:,} | {pc(tot[2], tot[0])} |")
    md += ["", "Measured vs estimate: the estimate counts a whole file when every construct the scanner sees maps",
           "to a supported feature (no type data, so it misses type errors and cascades). The measured number",
           "counts items that all three oracles take, so a file can be half in. A file with one unsupported",
           "construct loses only that item (the measured number can be above the estimate), but one early",
           "failure removes every user of the item in every file (below the estimate). The ranked root causes",
           "below name those failures.",
           "", "### Per top module", "", "| crate | module | universe | accepted | % | estimate | % |", "|---|---|---:|---:|---:|---:|---:|"]
    for (c, mod), (u, k, e, g) in sorted(per.items(), key=lambda kv: -kv[1][0]):
        md.append(f"| {c} | `{mod}` | {u:,} | {k:,} | {pc(k, u)} | {e:,} | {pc(e, u)} |")

    # Root causes over every slice's log would double count; use the main slice's log.
    roots = root_causes(main_m["removed"], lambda r: item_source(tsrust / SRC / r["file"], r["kind"], r["name"]))
    groups = defaultdict(lambda: {"roots": 0, "own": 0, "cascade": 0, "items": 0, "examples": []})
    for r in roots:
        g = groups[reason_key(r)]
        g["roots"] += 1
        g["own"] += r["own_lines"]
        g["cascade"] += r["cascade_lines"]
        g["items"] += 1 + r["cascade_items"]
        if len(g["examples"]) < 2:
            g["examples"].append(f"{r['file']} `{r['name']}`")
    ranked = sorted(groups.items(), key=lambda kv: -(kv[1]["own"] + kv[1]["cascade"]))
    md += ["", f"## Top {a.top} reasons lust (or rustc, or lust build) removed code from `{main_dir.name}`", "",
           "One row per root cause kind (oracle, code, message with numbers masked). `own` is the lines of the",
           "items the error was in; `cascade` is the lines of removals whose \"cannot find\" kind of error",
           "names one of those items, or a name the root's error named first (a std item lust lacks). Also",
           "cascades: a derive that fails on a field whose type went earlier, and a lust error that names the",
           "module (`ata`) or says \"module paths in types\" for a missing `mod::Item` (the removed item's",
           "source finds the Item). Removed trait impls have no name, so their users count as roots. Fix the",
           "top rows first. `external` rows are crates with no stand-in in support.rs, not lust errors. Lines",
           "can count twice when an item and its parent go in different rounds, so the columns sum to more",
           "than the slice lost.", "",
           "| # | oracle | code | message | roots | own lines | cascade lines | items | example |",
           "|---|---|---|---|---:|---:|---:|---:|---|"]
    for i, ((by, code, msg), g) in enumerate(ranked[:a.top], 1):
        msg_md = msg.replace("|", "\\|")
        md.append(f"| {i} | {by} | {code} | {msg_md} | {g['roots']} | {g['own']:,} | {g['cascade']:,} | {g['items']} | {'; '.join(g['examples'])} |")
    md += ["", "### Raw removed lines by oracle and code (prune.json, no cascade split)", "",
           "| oracle code | lines |", "|---|---:|"]
    for k, v in list(main_m["removed_lines_by_oracle_and_code"].items())[:20]:
        md.append(f"| {k} | {v:,} |")
    # Hand-written notes (grouped root causes, repro links) after NOTES_MARK survive a rerun.
    out = Path(a.out)
    old = out.read_text() if out.exists() else ""
    notes = old[old.index(NOTES_MARK):] if NOTES_MARK in old else ""
    out.write_text("\n".join(md) + "\n" + ("\n" + notes if notes else ""))
    print(f"wrote {a.out}: {tot[1]:,} of {tot[0]:,} universe lines accepted ({pc(tot[1], tot[0])}%)")


if __name__ == "__main__":
    main()
