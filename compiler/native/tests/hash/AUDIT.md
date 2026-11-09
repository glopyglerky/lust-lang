# K4 audit: FxHashMap iteration order in ts-rust

Date: 2026-10-12. Tree: `/home/theo/.t3/projects/ts-rust`, read only.
Method: `audit.py` in this directory (name-based static scan, limits
below), `lust port scan` counts, and hand checks of the risky sites.

## Result

The exact hashbrown table is necessary. The audit cannot prove that
Fx iteration order stays away from output. It found direct paths from
iteration order to logs, test data, Vec order, and build task order.
A sorting fix at some sites is possible in ts-rust itself, but lust
must not need ts-rust changes to stay byte-exact (docs/ARCH.md 3.5).
The C core in `src/runtime/hash.c` therefore matches hashbrown
0.17.1 exactly (see README.md in this directory).

The perturbed-FxHasher build from ARCH spike K4 (build all of ts-rust
with a `[patch]` hasher and diff baselines) was not run in this task.
Run it later as a second check. It can only find more sites; it cannot
remove the need that the sites below already show.

## Counts

From `lust port scan` on the ts-rust workspace (1,036 files):
`rustc-hash 2.1.2` and `indexmap 2.14.0` (its hashbrown is 0.17.1 in
Cargo.lock, the same version Rust 1.99 std vendors) are normal deps of
ts_goport, goport_lsproto, and goport_util. The built-in dependency
layer that covers FxHashMap and IndexMap has 1,787 findings in 350
files.

From `audit.py` (1031 Rust files under crates/):

- 2,166 `FxHashMap`/`FxHashSet` type mentions in 274 files.
- 17 type aliases name an Fx map or set (for example `ModeAwareCache`).
- 502 named Fx variables, fields, and parameters found (per file).
- 433 of those have no iteration site in their file: lookup-only use
  is the common case, and it is order-safe.
- 90 high-confidence iteration sites in production paths:
  73 `other` (for loops, collect, entry moves, retains, drains),
  10 sorted before use, 4 selection (first or find style),
  3 reduced (any, count, max of lengths).
  By form: 42 for-in loops, 21 `.iter()`, 7 `.into_iter()`,
  6 `.retain()`, 6 `.keys()`, 5 `.values()`, 2 `.values_mut()`,
  1 `.drain()`. Test paths add 10 more.
- For scale: `IndexMap` 537 mentions in 130 files, `IndexSet` 47 in
  21, `BTreeMap` 156 in 32. `FxIndexMap`/`FxIndexSet` aliases exist
  (`crates/ts_goport/src/core.rs:12`); their order is insertion order,
  so the IndexMap core, not the hash order, is the contract there.

These are lower bounds. The scan is name-based per file, so it misses
maps iterated through another file, through a return value, or under
a second name, and a few flagged sites share a name with an unrelated
Vec in the same file. Hand checks found those errors in both
directions (examples in Limits). The conclusion does not depend on
exact counts: one real output path is enough, and there are several.

## Riskiest sites (hand-checked)

- `crates/ts_goport/src/project/session.rs:2616`: iterates an
  FxHashSet (`ignored_paths`) and writes each path to the logger.
  Log byte order is Fx order.
- `crates/ts_goport/src/project/configfileregistry.rs:228` and `:250`:
  test entry structs collect three FxHashSets into `Vec` fields, and
  also iterate the `configs` FxHashMap per entry. Test comparisons
  see Fx order.
- `crates/ts_goport/src/project/projectcollectionbuilder.rs:638`:
  pushes paths from an Fx set iteration into a returned Vec.
- `crates/ts_goport/src/execute/build/orchestrator.rs:515`: runs
  `setup_build_task` in Fx set order. Build and emit order can
  follow it.
- `crates/ts_goport/src/api/session_p1.rs:2234`: walks `open_files`
  to build an API snapshot request. Project load order can follow.
- `crates/ts_goport/src/execute/build/shared_outputs.rs:204` and
  `:240`: an Fx set becomes `root_dirs` Vecs used for output paths.
- `crates/ts_goport/src/ls/autoimport/registry.rs:1173` and `:1209`:
  set order becomes the order buckets are scanned in. This feeds LSP
  auto-import state.
- `crates/ts_goport/src/checker/checker_p18.rs:608` plus checker
  service loops (`checker/services.rs:572`): removals and link writes
  in Fx order. The writes are by key, so content is safe today, but
  the same loops are one edit away from order-visible behavior.

Two patterns make static proof hard, and both occur often:

- Transitive order: iterating set A to insert into map B makes the
  bucket order of B depend on the order of A when keys collide.
- Erase order inside `retain` and `drain` changes which buckets
  become EMPTY vs DELETED. Later capacity and iteration follow it.

## Sites that are safe

Diagnostics are mostly safe: program diagnostics pass through
`filter_and_sort_diagnostics` and `sort_and_deduplicate_diagnostics`
(`crates/ts_goport/src/program.rs:3933`, `:4044`) before output, and
`errors_by_file` in `execute/tsc/diagnostics.rs` is only read by key
and for a max. Set unions, membership builds, counts, and writes by
key are safe. `SymbolTable` is not Fx at all: it is a custom
insertion-order table (`crates/ts_goport/src/core.rs:2266`), so
checker member order does not come from FxHashMap.

## Limits

Examples of scan error found by hand: `private_names` in `core.rs`
is a Vec, not the Fx set of the same name in that file; `files` in
`execute/incremental/program_to_snapshot.rs` is a slice from
`source_files()`, not its Fx set field; `node.parameters()` matched
the name `parameters`. In the other direction, the scan first missed
`for (path, entry) in &c.configs` (dotted receiver), now covered.
Treat site lists from `audit.py` as samples to check by hand, not as
proof about any one site.
