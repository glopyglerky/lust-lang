# Owned ts-rust kernels (milestone 4 gate fixtures)

Standalone crates copied from ts-rust (read-only at
`/home/theo/.t3/projects/ts-rust`, crate `ts_goport`). Each kernel is
one binary with a driver `main` and no dependencies. The milestone 4
gate in docs/PLAN.md is: ts-rust's scanner and JSON kernels compile
as owned fixtures, with the same bytes as rustc. Expected output in
each `expected/` dir is from rustc today.

Run everything with `scripts/test-kernels.sh` (rustc validates;
`LUST=path/to/lust` reports what lust does not support yet and does
not fail).

## Size tiers

Three kernels by code size. Each also runs inputs in three sizes
(small, medium, large). Line counts include driver and support code.

| Tier | Kernel | Lines | ts-rust source |
|---|---|---|---|
| small | gostring | 1635 | `src/gostring.rs` (bottom half of `scanner_util`) |
| medium | json | 7096 | `src/frontend/json.rs`, `json_ext.rs`, `json_indexmap.rs`, `gostd/{strconv,errors}.rs` |
| full | scanner | 14410 | `src/frontend/scanner/*`, `src/scanner_util.rs` items, `astdata`, `flags` |

Each kernel also has a single-file amalgam variant (`<name>-amalgam/
src/main.rs`, gostring 1643 lines, json 7118, scanner 14455) built
by `amalgamate.py`. Regenerate after any kernel edit:

```sh
python3 tests/kernels/amalgamate.py tests/kernels/scanner tests/kernels/scanner-amalgam/src/main.rs
```

## Input tiers

| Kernel | Small | Medium | Large | Error inputs |
|---|---|---|---|---|
| json | small.json, shapes.json (written) | tsconfig.base.json, 2177 B | effectDiagnosticMessages.json, 29336 B | invalid-syntax, invalid-trailing-comma |
| scanner | small.ts (written), shebang.ts | lib.es2015.iterable.d.ts, 18185 B | lib.es2023.array.d.ts, 40197 B | invalid.ts |
| gostring | fixed case table in the driver (7 byte-string cases) | - | - | invalid UTF-8 and surrogate cases are in the table |

Real inputs come from ts-rust test data and libs. Output is
deterministic: drivers never print hash map iteration order (map
contents print sorted or through the ordered map), and thread and
timer order never reaches stdout.

## How faithful the copies are

Copied files keep their ts-rust module paths (`crate::frontend::…
`, `crate::scanner_util`, `crate::diag`), mirrored by small `mod`
files so file bodies stay byte-identical. Every edit inside a copied
file is marked `KERNEL-EDIT` (13 marks today, mostly one-line
imports that bind a crate name to a local stand-in). No file in any
kernel uses `unsafe`; the ts-rust originals of these files do not
either.

Verbatim:

- gostring: `gostring.rs` whole.
- json: `json.rs`, `json_ext.rs`, `json_indexmap.rs`, `gostring.rs`,
  `gostd/strconv.rs`, `gostd/strconv_isprint.rs`, `gostd/errors.rs`
  whole.
- scanner: `scanner_p1.rs`, `scanner_p2.rs`, `regexp.rs`,
  `unicode_properties.rs`, `comment_ranges.rs`,
  `astdata/syntax_kind.rs`, `astdata/text.rs`, `flags.rs`,
  `flags_macros.rs`, `gostring.rs` whole.

Extractions (compiler-guided item copies, doc comments kept):

- `scanner/src/scanner_util.rs`: the 30 items the scanner uses,
  from the 4657-line `scanner_util.rs`, plus the Unicode ES
  identifier tables. AST items are not used by the scanner and are
  not extracted.
- `scanner/src/diagnostics.rs`: `Category`, `Message`, `FormatError`
  from `diagnostics/mod.rs` (one marked edit: `go_panic` becomes
  `panic!`).
- `scanner/src/diag.rs`: the 70 catalog entries the scanner names,
  from `diag.rs` plus `diagnostics/catalog.rs` (19942 lines in full).
- `scanner/src/core.rs`: `core.TextRange` (from `ast/node.rs`) and
  the `args!` macro (from `core.rs`).
- `scanner/src/program.rs`: `CommentDirective`, `CommentRange`, and
  the `NodeFactory::new_comment_range` constructor.
- `scanner/src/options.rs`: `ScriptTarget::string` and its `Display`.
- Not extracted: `frontend/scanner/scanner_ls.rs` (language-service
  helpers over AST nodes; the scanner core does not use it).

## Replaced external crates

`support.rs` in each kernel is the same file; keep the copies in
sync. ts-rust pins: memchr 2.8.2, rustc-hash 2.1.2, indexmap 2.14.0.

| Crate | Stand-in | Behavior note |
|---|---|---|
| memchr | naive byte search, same API incl. `memmem::Finder` | same positions; speed differs, results do not |
| rustc-hash | FxHash (rotate 5, xor, multiply) over std maps; `FxBuildHasher` unit struct | membership only; drivers never print map order |
| indexmap | Vec-backed ordered map and set | insertion order kept (JSON output depends on it); equality order-free, as in the crate |
| num-bigint, num-traits, ryu-js (via ts-rust `jsnum`) | `scanner/src/jsnum.rs`: f64 parse (correctly rounded), ECMAScript Number text over Rust shortest digits, limb radix conversion for bigint literals | a shortest-digit tie can in principle print a different last digit than ryu-js; no kernel input is known to tie |

Also replaced: the ts-rust `unported!` macro (same panic text, no
recording) and the bump arena behind `ArenaBox` in tests/compat
(Box::leak, same drop rule).

## Feature lists (lust-port, 2026-10-12)

Recorded with `cargo run --release --manifest-path
tools/lust-port/Cargo.toml -- scan --manifest-path
tests/kernels/<kernel>/Cargo.toml`. All six crates scan with zero
parse errors, zero missing modules and zero dependencies. Amalgam
scans match their kernels (findings differ by at most 21, from how
the scanner resolves stand-in crate paths in one file).

Findings by PLAN milestone (count of unsupported uses):

| Kernel | m4 | m5 | m6 | m7 | Files fully covered after |
|---|---|---|---|---|---|
| gostring (1635 lines) | 230 | 109 | 25 | 0 | m5: 1/3, m6: 3/3 |
| json (6954 prod lines) | 908 | 348 | 100 | 8 | m4: 2/10, m5: 4/10, m6: 8/10, m7: 10/10 |
| scanner (12049 prod lines) | 2837 | 247 | 125 | 4 | m4: 7/20, m5: 8/20, m6: 17/20, m7: 20/20 |

Coverage counts production files; generated table files
(`syntax_kind.rs`, `diag.rs`, `strconv_isprint.rs`) are outside its
universe. A file counts only when every feature in it is supported,
so whole-file coverage lags the findings curve: named lifetimes
(m6) sit in almost every file.

### gostring features, by milestone then count

| Feature | Milestone | Uses |
|---|---|---|
| core.tuples_arrays_slices | 4 | 126 |
| core.patterns_control | 4 | 64 |
| core.items | 4 | 11 |
| core.numbers_format | 4 | 9 |
| modules.imports | 4 | 8 |
| core.result_try | 4 | 3 |
| core.std_basics | 4 | 3 |
| macros.std_missing | 4 | 3 |
| core.derives_extra | 4 | 2 |
| macros.user | 4 | 1 |
| closures.general | 5 | 38 |
| iterators.chains | 5 | 34 |
| traits.generics | 5 | 23 |
| std.rc_refcell | 5 | 8 |
| dyn.trait | 5 | 4 |
| traits.impl_trait_arg | 5 | 2 |
| lifetimes.named | 6 | 15 |
| lifetimes.static_anon | 6 | 10 |

### json features, by milestone then count

| Feature | Milestone | Uses |
|---|---|---|
| core.tuples_arrays_slices | 4 | 390 |
| core.patterns_control | 4 | 192 |
| core.result_try | 4 | 190 |
| modules.imports | 4 | 43 |
| core.items | 4 | 36 |
| core.numbers_format | 4 | 18 |
| macros.std_missing | 4 | 17 |
| core.derives_extra | 4 | 7 |
| attrs.cfg | 4 | 6 |
| core.std_basics | 4 | 5 |
| macros.user | 4 | 2 |
| modules.path_attr | 4 | 2 |
| traits.generics | 5 | 114 |
| closures.general | 5 | 101 |
| iterators.chains | 5 | 72 |
| dyn.trait | 5 | 16 |
| std.rc_refcell | 5 | 16 |
| dyn.any | 5 | 11 |
| traits.impl_trait_arg | 5 | 10 |
| std.lazy_once_lock | 5 | 4 |
| closures.hrtb_bounds | 5 | 2 |
| ext.builtin_crates | 5 | 2 |
| lifetimes.static_anon | 6 | 55 |
| lifetimes.named | 6 | 45 |
| threads.send_sync_bounds | 7 | 6 |
| threads.sync | 7 | 2 |

### scanner features, by milestone then count

| Feature | Milestone | Uses |
|---|---|---|
| core.tuples_arrays_slices | 4 | 1820 |
| core.items | 4 | 562 |
| core.patterns_control | 4 | 153 |
| macros.user | 4 | 109 |
| modules.imports | 4 | 63 |
| macros.std_missing | 4 | 51 |
| core.numbers_format | 4 | 36 |
| core.derives_extra | 4 | 15 |
| attrs.cfg | 4 | 12 |
| core.result_try | 4 | 8 |
| core.std_basics | 4 | 7 |
| modules.path_attr | 4 | 1 |
| closures.general | 5 | 86 |
| iterators.chains | 5 | 77 |
| traits.generics | 5 | 42 |
| traits.impl_trait_arg | 5 | 14 |
| std.rc_refcell | 5 | 10 |
| closures.hrtb_bounds | 5 | 6 |
| dyn.trait | 5 | 6 |
| std.lazy_once_lock | 5 | 6 |
| lifetimes.static_anon | 6 | 64 |
| lifetimes.named | 6 | 61 |
| threads.sync | 7 | 4 |

### What this says to build first

For W-CORELANG and W-STD-A, in order:

1. Milestone 4 core first. It is 63% of gostring findings, 67% of
   json, 88% of scanner. Within it: tuples, arrays, slices and
   indexing (the top item in all three), then patterns and control,
   then const and static items (562 uses in the scanner: token and
   Unicode tables), then `Result` and `?` (190 json uses), then
   `macro_rules!` (109 scanner uses: `go_flags!`, `go_enum!`,
   `args!`) and the std macros.
2. Closures and iterators next, before full traits. They are the
   largest m5 blocks in gostring and scanner, and the json kernel
   needs traits generics (114) with them because its whole API is
   two traits. `dyn Any` downcast (11 uses, json only) and the
   blanket impl shape are the trait items that matter first; see
   tests/compat/blanket_anyvalue.rs.
3. Named lifetimes gate whole files in all three kernels even
   though the use counts are small. The scanner state borrows the
   source text (`Scanner<'a>`, token values as `&'a str`) and the
   JSON decoder borrows input names (`Cow<'a, str>`). As DESIGN-TS
   said, the verbatim kernels need the later lifetime work; a
   milestone-4-only subset of these fixtures does not close. The
   first kernel that can go green end to end is gostring (no m7
   items, 25 lifetime uses).

## Known limits

- The `#[cfg(test)]` modules inside copied files (for example in
  `json.rs`) are inert: the gate builds and runs the drivers, it
  does not run `cargo test` inside kernels.
- FxHashMap iteration order in the stand-in is not promised to
  match rustc-hash 2.1 across capacities. Kernel output never
  depends on it by construction.
- Expected outputs embed the stand-ins' behavior. If a stand-in is
  ever found to differ from its crate on a kernel input, fix the
  stand-in and regenerate the expected files in the same commit.
