#!/usr/bin/env python3
"""Generate the ARCH 4.4 layout shape corpus.

The generator enumerates the supported-shape grammar of ARCH 4.4
with the bounds below. It is not random: every shape inside the
bounds is emitted, in a fixed order, so a diff always means the
grammar or a bound changed on purpose.

Bounds:
- FIELD_KINDS: every field kind the gate names (scalars of each
  width, refs, boxes, small enums, bool and char fields,
  repr-int enums, small arrays), plus NonZero and fn pointers
  for niche coverage.
- Structs: 1 field of every kind; 2 fields, every ordered pair
  of distinct kinds; 3 fields, every ordered triple of distinct
  kinds from CORE_KINDS.
- Tuples: every ordered pair of distinct kinds, as print lines.
- Enums: at most 3 variants. One payload variant (1 or 2 fields
  from CORE_KINDS) plus one empty variant; two payload variants
  of one CORE_KINDS field each; one payload variant plus two
  empty variants for CORE_KINDS singles.
- Wraps: a generated 2-field struct from CORE_KINDS inside one
  more struct with one extra CORE_KINDS field (nesting depth 2).
- Nested Option: depth 1 and 2 over every generated name whose
  fields come only from CORE_KINDS, as print lines.
- Extra families (EXTRA_DEFS, each also printed under Option
  depth 1 and 2): repr(C) and repr(C, int) data enums with
  mixed-alignment payloads, repr(int) data enums for contrast,
  array payloads (bool, char, nested Option elements, empty
  arrays), u128 payloads next to niche-carrying variants, and
  fieldless enums with negative or wrap-edge discriminants.
- Direct prints (EXTRA_PRINTS): Option over array types.

Usage:
  generate.py            check the committed files are current
  generate.py --write    rewrite pass/gen_shapes.rs and record
                         pass/gen_shapes.stdout from rustc
"""
import itertools
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT_RS = HERE / "pass" / "gen_shapes.rs"
OUT_STDOUT = HERE / "pass" / "gen_shapes.stdout"

# kind name -> Rust type text
FIELD_KINDS = [
    ("u8", "u8"),
    ("i16", "i16"),
    ("u32", "u32"),
    ("u64", "u64"),
    ("u128", "u128"),
    ("bool", "bool"),
    ("char", "char"),
    ("f64", "f64"),
    ("ref", "&'static u8"),
    ("box", "Box<u32>"),
    ("nonzero", "std::num::NonZeroU32"),
    ("fnptr", "fn() -> u8"),
    ("smallenum", "SmallEnum"),
    ("reprenum", "ReprEnum"),
    ("arr3", "[u8; 3]"),
    ("arr16x2", "[u16; 2]"),
]
CORE_KINDS = ["u8", "u32", "u64", "bool", "ref", "box", "arr3"]
KIND = dict(FIELD_KINDS)

# Explicit extra families: full type definitions, each printed plain
# and under Option depth 1 and 2. Bounded by this list.
EXTRA_DEFS = [
    # repr(C) data enums: tag plus a union of repr(C) variant structs.
    "#[repr(C)] enum XC1 { A(u8, u64), B }",
    "#[repr(C)] enum XC2 { A(u64, u8), B(u8) }",
    "#[repr(C)] enum XC3 { A(i16, f64) }",
    "#[repr(C)] enum XC4 { A(bool, u16, [u8; 3]), B([u64; 2]) }",
    "#[repr(C)] enum XC5 { A(u128, bool), B(u8) }",
    "#[repr(C, u8)] enum XC6 { A(bool, u8), B(Option<char>, u128, u128) }",
    "#[repr(C, u8)] enum XC7 { A(bool, u16, [(i32,); 0]) }",
    "#[repr(C, u32)] enum XC8 { A(u8, u128), B }",
    "#[repr(C, i64)] enum XC9 { A(u8), B(u64, u64) }",
    # repr(int) alone: tag first inside each variant struct.
    "#[repr(u8)] enum XI1 { A(bool, u8), B(Option<char>, u128, u128) }",
    "#[repr(u8)] enum XI2 { A(u8, u64), B }",
    "#[repr(i128)] enum XI3 { A(u8) = -5, B = 100 }",
    "#[repr(u128)] enum XI4 { A = 5, B }",
    # Array payloads and their element niches.
    "enum XA1 { A([bool; 2]), B(u8) }",
    "enum XA2 { A([char; 3]), B }",
    "enum XA3 { A([bool; 4]), B(u8, u8, u8) }",
    "enum XA4 { A([Option<f64>; 3]), B(usize), C(u64, bool) }",
    "enum XA5 { A([bool; 0], u32), B(u8) }",
    "enum XA6 { A(([bool; 1], u8)), B }",
    # u128 payloads beside niche-carrying variants (align-16 fit).
    "enum XU1 { A(u128), B([u64; 2], (bool, u64)) }",
    "enum XU2 { A((), u128), B([f64; 2], (Option<Option<i32>>, i64)) }",
    "enum XU3 { A(u128), B(u64, u64, bool, u64) }",
    "enum XU4 { A(u32), B([u8; 4], (bool, u32)) }",
    # Fieldless enums with negative or wrap-edge discriminants.
    "enum XN1 { A = -1, B = 127 }",
    "enum XN2 { A = -128, B = 127 }",
    "enum XN3 { A = -2, B = -1 }",
    "enum XN4 { A = 0, B = 255 }",
    "enum XN5 { A = -100, B = 100 }",
    "#[repr(i8)] enum XN6 { A = -1, B = 0 }",
    "enum XN7 { A = 200, B = 201 }",
]

# Direct type prints with no definition of their own.
EXTRA_PRINTS = [
    ("OPT_arr_bool_2", "Option<[bool; 2]>"),
    ("OPT_arr_char_3", "Option<[char; 3]>"),
    ("OPT_arr_optbool_2", "Option<[Option<bool>; 2]>"),
    ("OPT_arr_bool_0", "Option<[bool; 0]>"),
    ("OPT2_arr_bool_3", "Option<Option<[bool; 3]>>"),
    ("ARR_u128_2", "[u128; 2]"),
    ("TUP_unit_u128", "((), u128)"),
]


def build_source() -> str:
    defs: list[str] = []
    prints: list[str] = []  # (label, type text)
    core_names: list[tuple[str, str]] = []  # named types over CORE_KINDS only

    defs.append("enum SmallEnum { A(u8), B }")
    defs.append("#[repr(u64)] enum ReprEnum { X = 0, Y = 1, Z = 2 }")

    def add_struct(kinds: list[str]) -> str:
        name = "S_" + "_".join(kinds)
        fields = ", ".join(f"pub f{i}: {KIND[k]}" for i, k in enumerate(kinds))
        defs.append(f"struct {name} {{ {fields} }}")
        prints.append((name, name))
        if all(k in CORE_KINDS for k in kinds):
            core_names.append((name, name))
        return name

    def add_enum(name: str, variants: list[tuple[str, list[str]]]) -> None:
        body = ", ".join(
            f"{v}({', '.join(KIND[k] for k in ks)})" if ks else v for v, ks in variants
        )
        defs.append(f"enum {name} {{ {body} }}")
        prints.append((name, name))
        if all(k in CORE_KINDS for _, ks in variants for k in ks):
            core_names.append((name, name))

    all_kinds = [k for k, _ in FIELD_KINDS]
    for k in all_kinds:
        add_struct([k])
    for a, b in itertools.permutations(all_kinds, 2):
        add_struct([a, b])
        prints.append((f"TUP_{a}_{b}", f"({KIND[a]}, {KIND[b]})"))
    for triple in itertools.permutations(CORE_KINDS, 3):
        add_struct(list(triple))

    for k in CORE_KINDS:
        add_enum(f"E1_{k}", [("A", [k]), ("B", [])])
        add_enum(f"E1x_{k}", [("A", [k]), ("B", []), ("C", [])])
    for a, b in itertools.permutations(CORE_KINDS, 2):
        add_enum(f"E2f_{a}_{b}", [("A", [a, b]), ("B", [])])
        add_enum(f"E2v_{a}_{b}", [("A", [a]), ("B", [b])])

    wrap_names = []
    for a, b in itertools.permutations(CORE_KINDS, 2):
        inner = f"S_{a}_{b}"
        for extra in CORE_KINDS:
            name = f"W_{a}_{b}_{extra}"
            defs.append(f"struct {name} {{ pub inner: {inner}, pub extra: {KIND[extra]} }}")
            prints.append((name, name))
            wrap_names.append(name)

    for label, ty in core_names:
        prints.append((f"OPT_{label}", f"Option<{ty}>"))
        prints.append((f"OPT2_{label}", f"Option<Option<{ty}>>"))

    defs.extend(EXTRA_DEFS)
    for d in EXTRA_DEFS:
        name = d.split("enum ", 1)[1].split(" ", 1)[0]
        prints.append((name, name))
        prints.append((f"OPT_{name}", f"Option<{name}>"))
        prints.append((f"OPT2_{name}", f"Option<Option<{name}>>"))
    prints.extend(EXTRA_PRINTS)

    lines = [
        "// Generated by tests/adapters/layout/generate.py. Do not edit.",
        "// ARCH 4.4 shape grammar enumeration; see generate.py for bounds.",
        "#![allow(dead_code)]",
        "use std::mem::{align_of, size_of};",
        "",
        *defs,
        "",
        "fn p<T>(name: &str) { println!(\"{name} size={} align={}\", size_of::<T>(), align_of::<T>()); }",
        "fn main() {",
        *[f'    p::<{ty}>("{label}");' for label, ty in prints],
        "}",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    source = build_source()
    if "--write" in sys.argv:
        OUT_RS.write_text(source)
        with tempfile.TemporaryDirectory() as tmp:
            exe = Path(tmp) / "gen_shapes"
            subprocess.run(
                ["rustc", "--edition", "2024", "-C", "panic=abort",
                 "-C", "overflow-checks=on", "-A", "warnings",
                 "-o", str(exe), str(OUT_RS)],
                check=True,
            )
            out = subprocess.run([str(exe)], check=True, capture_output=True, text=True)
        OUT_STDOUT.write_text(out.stdout)
        print(f"generate: wrote {OUT_RS.name} ({out.stdout.count(chr(10))} shapes)")
        return 0
    current = OUT_RS.read_text() if OUT_RS.exists() else ""
    if current != source:
        print("generate: pass/gen_shapes.rs is stale; run generate.py --write")
        return 1
    print("generate: gen_shapes.rs is current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
