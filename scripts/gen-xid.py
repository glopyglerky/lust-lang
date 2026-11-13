#!/usr/bin/env python3
"""Generate src/xid_table.rs: XID and NFC tables for the lexer.

The tables come from the Unicode Character Database (UCD) of the
Unicode version that rustc 1.99 uses. rustc checks identifiers with
the unicode-ident crate; unicode-ident 1.0.24 and the rustc 1.99
standard library (char::UNICODE_VERSION) both use Unicode 17.0.0, and
rustc 1.99 rejects identifier chars that Unicode 18 added to XID
(U+0558, U+209D, U+A7DD), so Unicode 17.0.0 is the right version.

Data used, all from https://www.unicode.org/Public/17.0.0/ucd/:
  DerivedCoreProperties.txt   XID_Start and XID_Continue ranges.
  UnicodeData.txt             Canonical combining class (ccc) and
                              canonical decomposition mappings.
  DerivedNormalizationProps.txt  Full_Composition_Exclusion, to
                              derive the NFC composition map.

Hangul decomposition and composition are algorithmic in the lexer
and need no table.

Usage: scripts/gen-xid.py [--ucd-dir DIR] [--out FILE]
With no --ucd-dir, the three UCD files are downloaded into a
temporary directory. Run this after a rustc update that changes the
Unicode version, and update UNICODE_VERSION below to match.
"""

import argparse
import pathlib
import sys
import tempfile
import urllib.request

UNICODE_VERSION = "17.0.0"
UCD_BASE = f"https://www.unicode.org/Public/{UNICODE_VERSION}/ucd"
FILES = ["DerivedCoreProperties.txt", "UnicodeData.txt", "DerivedNormalizationProps.txt"]


def parse_range(text):
    """A UCD code point field: `00C0` or `00C0..00D6`."""
    text = text.strip()
    if ".." in text:
        lo, hi = text.split("..")
        return int(lo, 16), int(hi, 16)
    cp = int(text, 16)
    return cp, cp


def read_props(path, wanted):
    """Code point ranges per property from a semicolon UCD file."""
    out = {name: [] for name in wanted}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        fields = [f.strip() for f in line.split(";")]
        if len(fields) >= 2 and fields[1] in out:
            out[fields[1]].append(parse_range(fields[0]))
    return out


def merge_ranges(ranges):
    """Sort ranges and merge overlaps and neighbours."""
    out = []
    for lo, hi in sorted(ranges):
        if out and lo <= out[-1][1] + 1:
            out[-1][1] = max(out[-1][1], hi)
        else:
            out.append([lo, hi])
    return [(lo, hi) for lo, hi in out]


def read_unicode_data(path):
    """ccc and canonical decomposition per code point.

    Returns (ccc_ranges, decomp) where decomp maps a code point to a
    tuple of one or two code points. Compatibility decompositions
    (field 5 starts with `<`) are ignored: NFC uses canonical ones.
    """
    ccc = {}
    decomp = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        fields = line.rstrip("\n").split(";")
        if len(fields) < 6:
            continue
        cp = int(fields[0], 16)
        value = int(fields[3])
        if value != 0:
            ccc[cp] = value
        mapping = fields[5].strip()
        if mapping and not mapping.startswith("<"):
            parts = tuple(int(p, 16) for p in mapping.split())
            if len(parts) > 2:
                sys.exit(f"gen-xid: canonical decomposition of U+{cp:04X} has {len(parts)} chars; the table format holds at most 2")
            decomp[cp] = parts
    # Merge consecutive code points that share a ccc into one range.
    ccc_ranges = []
    for cp in sorted(ccc):
        if ccc_ranges and cp == ccc_ranges[-1][1] + 1 and ccc[cp] == ccc_ranges[-1][2]:
            ccc_ranges[-1][1] = cp
        else:
            ccc_ranges.append([cp, cp, ccc[cp]])
    return [(a, b, v) for a, b, v in ccc_ranges], decomp, ccc


def emit_ranges(out, name, doc, ranges, fmt):
    out.append(f"/// {doc}")
    out.append(f"pub(crate) static {name}: &[{fmt[0]}] = &[")
    line = "    "
    for entry in ranges:
        text = fmt[1](entry)
        if len(line) + len(text) + 1 > 138:
            out.append(line)
            line = "    "
        line += text + " "
    if line.strip():
        out.append(line)
    out.append("];")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    root = pathlib.Path(__file__).resolve().parent.parent
    ap.add_argument("--ucd-dir", type=pathlib.Path, help="directory holding the three UCD files (default: download)")
    ap.add_argument("--out", type=pathlib.Path, default=root / "src" / "xid_table.rs")
    args = ap.parse_args()

    tmp = None
    if args.ucd_dir is None:
        tmp = tempfile.TemporaryDirectory(prefix="lust-ucd-")
        ucd = pathlib.Path(tmp.name)
        for name in FILES:
            print(f"gen-xid: downloading {UCD_BASE}/{name}")
            urllib.request.urlretrieve(f"{UCD_BASE}/{name}", ucd / name)
    else:
        ucd = args.ucd_dir
        for name in FILES:
            if not (ucd / name).is_file():
                sys.exit(f"gen-xid: missing {ucd / name}; download it from {UCD_BASE}/{name}")

    xid = read_props(ucd / "DerivedCoreProperties.txt", ["XID_Start", "XID_Continue"])
    xid_start = merge_ranges(xid["XID_Start"])
    xid_continue = merge_ranges(xid["XID_Continue"])
    ccc_ranges, decomp, ccc = read_unicode_data(ucd / "UnicodeData.txt")
    excl = read_props(ucd / "DerivedNormalizationProps.txt", ["Full_Composition_Exclusion"])
    excluded = set()
    for lo, hi in excl["Full_Composition_Exclusion"]:
        excluded.update(range(lo, hi + 1))

    # NFC composition map: canonical decompositions of length two
    # whose first char is a starter (ccc 0), unless the composite is
    # a full composition exclusion (UAX #15, Composition_Map).
    compose = []
    for cp, parts in sorted(decomp.items()):
        if len(parts) == 2 and ccc.get(parts[0], 0) == 0 and cp not in excluded:
            if ccc.get(cp, 0) != 0:
                sys.exit(f"gen-xid: composite U+{cp:04X} has nonzero ccc; the lexer assumes composites are starters")
            compose.append((parts[0], parts[1], cp))
    compose.sort()
    if len({(a, b) for a, b, _ in compose}) != len(compose):
        sys.exit("gen-xid: duplicate composition pair")

    decomp_rows = [(cp, parts[0], parts[1] if len(parts) == 2 else 0) for cp, parts in sorted(decomp.items())]

    out = [
        f"// XID and NFC tables for the lexer, generated by scripts/gen-xid.py from the",
        f"// Unicode {UNICODE_VERSION} Character Database (DerivedCoreProperties.txt,",
        f"// UnicodeData.txt, DerivedNormalizationProps.txt). Unicode {UNICODE_VERSION} is the",
        "// version rustc 1.99 uses for identifiers (the unicode-ident crate and",
        "// char::UNICODE_VERSION). Do not edit; regenerate after a rustc update.",
        "",
    ]
    pair = ("(u32, u32)", lambda r: f"(0x{r[0]:x}, 0x{r[1]:x}),")
    triple = ("(u32, u32, u32)", lambda r: f"(0x{r[0]:x}, 0x{r[1]:x}, 0x{r[2]:x}),")
    emit_ranges(out, "XID_START_RANGES", "Ranges of code points with the XID_Start property.", xid_start, pair)
    out.append("")
    emit_ranges(out, "XID_CONTINUE_RANGES", "Ranges of code points with the XID_Continue property.", xid_continue, pair)
    out.append("")
    emit_ranges(
        out,
        "CCC_RANGES",
        "Ranges with a nonzero canonical combining class: (first, last, class as u32).",
        ccc_ranges,
        ("(u32, u32, u32)", lambda r: f"(0x{r[0]:x}, 0x{r[1]:x}, {r[2]}),"),
    )
    out.append("")
    emit_ranges(
        out,
        "CANON_DECOMP",
        "Canonical decomposition mappings: (char, first, second); second is 0 for a one-char mapping. Hangul syllables decompose in code.",
        decomp_rows,
        triple,
    )
    out.append("")
    emit_ranges(
        out,
        "COMPOSE_PAIRS",
        "NFC composition map: (starter, char, composite), sorted by (starter, char). Hangul pairs compose in code.",
        compose,
        triple,
    )
    out.append("")

    args.out.write_text("\n".join(out) + "\n", encoding="utf-8")
    print(
        f"gen-xid: wrote {args.out}: {len(xid_start)} XID_Start ranges, {len(xid_continue)} XID_Continue ranges, "
        f"{len(ccc_ranges)} ccc ranges, {len(decomp_rows)} decompositions, {len(compose)} composition pairs"
    )
    if tmp is not None:
        tmp.cleanup()


if __name__ == "__main__":
    main()
