# bench/slices: pinned ts-rust slices for the speed benchmark

The 100x promise is measured here: warm `lust check` after a one-function edit,
against `cargo check` (rustc 1.99, incremental) on the same source.

A slice is real ts-rust source (commit in `prune.json`), copied verbatim, minus
whole items that lust cannot take yet. Both lust and rustc accept every slice,
and `lust build` builds it.

| Slice | Lines (code) | From |
|---|---|---|
| `tsrust-all4` | 165,204 (102,440) | the `tsrust-all` spec, pruned again with lust next 4f4c96f84 (W-BENCH4; 671,746 lines copied) |
| `tsrust-all` | 160,065 (98,983) | all of ts_goport (`lib.rs`) with both part crates mounted, plus smallvec and xxhash_rust stand-ins (671,746 lines copied) |
| `tsrust-full` | 150,147 (91,879) | all of ts_goport (`lib.rs`) with both part crates mounted (670,906 lines copied) |
| `tsrust-nochecker` | 107,084 (67,850) | ts_goport `lib.rs` with both part crates, without checker, ls, lsp, project, api, execute, cmd (406,266 lines copied) |
| `goport-util-lsp` | 53,918 (36,890) | ts-rust `goport_util` + `goport_lsproto` merged into one crate (147,484 lines) |
| `goport-util` | 37,897 (28,695) | ts-rust `goport_util` (87,785 lines in its module tree) |
| `goport-util-small` | 8,337 (5,014) | the same crate without gostd, locale, fswatch, astdata |

## How a slice is made

`prune.py <spec.json> <out-dir>` copies the crate's module tree from ts-rust
(read only), then repeats: `lust check`, remove the innermost item that holds
each error; `rustc`, the same; `lust build`, the same. It stops when all three
accept. A removed item makes its users fail to resolve, so they go too: the
slice is closed over its own dependencies. When lust gives an error with no
usable location (a std span under the root file, line 1, a panic), the fn is
found by stubbing fn bodies in halves (bisect). `prune.json` lists every
removed item with the oracle and error code that removed it. That list is a
ranked to-do list for lust coverage.

`items/` is a small syn tool that lists items, fn bodies, and the module tree
(`#[path]` resolved as rustc does: next to the file, or below an inline module).
`driver.rs` is the bench `main` body (tspath calls with printed results); the
benchmark mounts it in its work copy so build rows can compare program output
with rustc. Keep the fns it calls in every slice, or update it.

Bench-owned text, not ts-rust code:

- the root file (`slice_main.rs`): ts-rust's `goport_util_root.rs` with a
  `fn main() {}`, the stand-in mods, and the `root_replace` edits of the spec;
- `support.rs`: stand-ins for `rustc_hash`, `indexmap`, `memchr` (from
  tests/kernels, with two lust-friendly changes; see its header). Paths to
  those crates are rewritten to `crate::<name>::` (spec `rewrite`).
- `support_smallvec.rs` (lust's smallvec adapter, `smallvec!` as a module
  macro) and `support_xxhash.rs` (the xxh3 API over FNV-1a: other hash
  values than the real crate, the same in lust and rustc). `tsrust-all` and `tsrust-all4` only.
- in `goport-util-lsp`, `goport_util::` paths become `crate::` (spec
  `rewrite_to`): goport_lsproto's own root re-exports goport_util, so the
  merged crate keeps every path's meaning.

`core` (`gopanic.rs`) is mounted again: lust's std no longer breaks on a
user module named `core` (fixed in c6beebe29).

Bigger slices (`tsrust-*`) start from ts_goport's `lib.rs` and mount the two
part crates as modules (`#[path = "goport_util_root.rs"] pub mod goport_util;`).
Their files keep their `crate::` paths, which reach the same items through
ts_goport's re-exports; `goport_util::` and `goport_lsproto::` paths become
`crate::goport_util::` and `crate::goport_lsproto::`. A spec's `unmount` list
leaves top modules of `lib.rs` out, for a slice between 50k lines and full size.

Not stubbed: `smallvec` and `xxhash_rust` outside `tsrust-all`, and the other
external crates (users are pruned), and the data files that `include_bytes!` reads (prune.py copies
only `.rs` files). `bench/slices/coverage.py` counts both apart from lust errors.

## Grow a slice

```sh
cargo build --release
python3 bench/slices/prune.py bench/slices/goport-util/spec.json /tmp/gu
rm -rf bench/slices/goport-util/src && cp -r /tmp/gu/src /tmp/gu/prune.json bench/slices/goport-util/
```

Dev features: prune.py passes the package's effective dev feature list
to lust and to its rustc oracle (docs/dev-features.md). It reads the
lust config from the ts-rust tree, or `--dev-features a,b` /
`--dev-exclude-features a,b`, or `$LUST_DEV_FEATURES` /
`$LUST_DEV_EXCLUDE_FEATURES`, and records `dev_features` in prune.json.
ts-rust dev builds run with jemalloc off, so on a ts-rust tree without
a lust config pass `--dev-exclude-features jemalloc`. speed_slice.py
replays the recorded list for both compilers. With jemalloc off,
`program.rs` keeps its full 5,660 lines in a ts_goport slice (its
`tikv_jemalloc_ctl` blocks are cfg-off) and `jemalloc_layout.rs`
drops out as an unloaded file.

When a round stops on a lust error it cannot locate (a build failure outside fn bodies), add the
item to the spec's `drop` list (`[[file, item, reason]]`; prune.json records it with `"by": "spec"`)
and run again. On next 71f51f1d4 that is `frontend/module/cache.rs` `AheadCall` (W-BENCH4 repro
arc_slice_recursive_enum_build.rs).

Then check that the anchors in `edits.json` still occur once (the benchmark
fails with a clear message if not), and rerun the benchmark.

## Measure

```sh
python3 scripts/scale-gate.py --slice bench/slices/goport-util --json out.json
scripts/speed-row.sh            # the quick gate row (small slice)
```

Run timing on a quiet host (boat), never on a loaded shared box. Results and
the method are in docs/speed.md.
