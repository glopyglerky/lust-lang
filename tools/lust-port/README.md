# lust-port

`lust-port scan` is the gap scanner from PLAN.md milestone 3. It reads a
Rust crate with a full parser and reports every feature lust does not
support yet, grouped by root cause. Run it as `lust port scan` (the
`lust` client execs this binary) or directly.

This crate is standalone. It is not part of the root crate, and the root
crate gains no dependencies from it.

## Use

```sh
cargo build --release --manifest-path tools/lust-port/Cargo.toml
tools/lust-port/target/release/lust-port scan --manifest-path /path/to/Cargo.toml [-p pkg] [--target x86_64-unknown-linux-gnu]
tools/lust-port/target/release/lust-port scan --manifest-path /path/to/Cargo.toml -p pkg --json > report.json
tools/lust-port/target/release/lust-port scan --manifest-path /path/to/Cargo.toml -p pkg --markdown > report.md
```

- With `-p`, only that package is scanned. With a package manifest and
  no `-p`, that package is scanned. With a virtual workspace manifest
  and no `-p`, every workspace member is scanned.
- `--json` writes the full machine report. Add `--summary` to keep at
  most 5 findings per feature (counts and samples stay). The full
  ts-rust report is well over 2 MB, so `docs/ts-rust-gaps.json` is a
  summary.
- `--keys-json <file>` writes every raw scanner key with its count and
  the feature it maps to. A null feature means no rule: the coverage
  curve treats that construct as supported. Sort the null rows by
  count to find constructs that need a probe and a rule.
- The scan is read-only. It runs `cargo metadata --no-deps`, with
  `--locked` when a `Cargo.lock` exists, and only reads `.rs` files.
- A completed scan exits 0 even when gaps are found. Gaps are output.
- Cargo features: `cfg(feature = "...")` is evaluated with each
  package's effective dev features. The project lust config wins
  (`[package.metadata.lust]` or `[workspace.metadata.lust]`
  `dev-features` / `dev-exclude-features`, or a `.lust.toml` file; see
  `docs/dev-features.md` in the lust repo). With no config, the package
  `default` list applies, expanded through the `[features]` table.
  `--cargo-features a,b` (exact list) and
  `--cargo-exclude-features a,b` (removed from `default`) override the
  config for one scan. Cfg-false items, statements, fields, variants,
  and match arms are dropped from the AST before counting, and a
  cfg-false `mod` file is target-specific. The JSON report lists the
  effective features per package in `cargo_features`, and each
  dependency row says whether the dev selection enables it
  (`enabled`). Feature expansion is syntactic (metadata runs with
  `--no-deps`, so no resolve graph is read).

## What it reports

- One finding per primary-key hit: feature id, file, byte range, line,
  snippet, scope (active, test-only, or target-specific), fix class
  (compiler work, adapter, mechanical rewrite, manual), and whether a
  release-safe rewrite exists.
- Human output groups findings by root cause, then by feature, with at
  most 2 samples per feature.
- Coverage: the missing features per file, and how many prod files and
  lines would compile after each PLAN milestone and after each
  INVENTORY.md step.
- Dependencies: each direct dependency of the scanned packages with
  the dev-path decision from `dependencies.json` (built-in adapter,
  C shim, or compile from source). Release always uses Cargo.
- Parse errors, missing modules, and cfg predicates that cannot be
  decided from the target triple and the effective features (custom
  cfg flags and `target_feature`).

## Tables and how they stay true

- `features.json` maps raw scanner keys to a feature id, lust status,
  PLAN milestone, inventory step, root cause, and fix class. First
  match wins. `primary_keys` create findings; `keys` block a file in
  the coverage curve. A supported feature has milestone 0 and no
  primary keys.
- Statuses are measured. `tests/features/` holds one small program per
  feature id (a `_b` file probes a second form), in the form ts-rust
  writes. `python3 scripts/port-scan-sync.py` from the repo root runs
  each probe under rustc 1.99 and under `lust check` plus `lust run`
  and prints one status per feature. Add `--write` to store status and
  milestone in `features.json` (notes stay hand-written), and
  `--coverage` to rescan ts-rust into `docs/ts-rust-gaps.json` and
  `docs/ts-rust-coverage.md`. Run both after each wave. When one form
  of a feature passes and another fails, split the failing keys into a
  new feature when the scanner key tells the forms apart; else set
  `partial` and name the forms in the note. A partial feature still
  blocks files in the curve.
- A later `lust capabilities --json` can feed the same ids:
  `lust-port scan --capabilities caps.json` accepts
  `{"features": [{"id": "...", "status": "...", "milestone": 0}]}` and
  overrides the table by id.
- `--features <file>` and `--dependencies <file>` replace the tables.

## Limits

The scanner has no type information. Method names are not tied to
receivers, so `method.*` keys stay unruled as a class: a missing std
method on a supported type does not block a file here. Paths resolve
through `use` items and crate preludes, `macro_rules!` bodies are not
expanded, and user macro arguments count only when they parse as
expressions. A construct with no rule in `features.json` is treated
as supported, so a new std or crate path needs a rule; use
`--keys-json` to find unruled keys. The code starts from the ts-rust inventory analyzer
(`/tmp/lust-ts/inventory/analyzer`), so its raw keys and counts match
that study by construction; see `docs/ts-rust-gaps.md` for the
comparison.
