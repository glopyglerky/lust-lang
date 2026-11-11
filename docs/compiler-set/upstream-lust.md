# Preserve upstream provenance and working fork changes

The native compiler is an evolving Lust fork. Keep upstream history and source
provenance while retaining the Lustr changes required for correct behavior.
Restoring a pristine upstream tree is not a valid repair if it deletes working
fork functionality. The [native compiler contract](native-contract.md) remains
the full-project goal.

## Source identities

[`upstream/lust.pin`](../../upstream/lust.pin) records two different identities:

- Upstream `t3dotgg/lust-lang` revision
  `240e08d91d4aed0299639f6f11fd522912b248f9`, tree
  `0ab1b09815802894e6078de0af59273cbdf46cb0`.
- The reviewed native fork tree at `compiler/native`,
  `5a1e07fd605e1267c16e9f293691e470e6840107`.

The fork retains upstream compiler and library source, fixtures, licenses,
research probes and metadata. Its active source includes the fixes below.
The original upstream README remains in the component; the root README
describes this fork. External dependencies and generated artifacts require
their own qualification.

## Recovery of removed functionality

Commit [`82d7753`](https://github.com/glopyglerky/lustr/commit/82d77539680fec9cf822ca8231d1abb985069c48)
replaced the active native source with the pristine upstream tree. That removed
working changes present at parent
[`fb4c4bc`](https://github.com/glopyglerky/lustr/commit/fb4c4bc79a1d17f1c0f7cdcb47dced6d0f88c316).
The repair restores those implementation blobs exactly:

| Paths | Restored behavior |
| --- | --- |
| `src/image.rs` | Linux-only image startup and personality calls; child processes do not inherit the private arena setting. |
| `src/client.rs`, `src/daemon.rs`, `src/image.rs` | Correct variadic `fcntl` declarations, preserving the actual C ABI. |
| `src/driver/buildscript.rs` | Current and legacy Cargo output layouts; missing/unreadable output is an error; LBS5 invalidates older incomplete records. |
| `src/driver/cache.rs` | Token-aware detection of external-input macros, including whitespace/comments and raw identifiers. |
| `build.rs` | Build identity includes lockfile, build recipe, configuration and actual compiler identity. |
| `Cargo.toml`, `Cargo.lock` | `lustr-native` package identity and separation from the coordinator workspace. |
| `scripts/ci.sh`, `scripts/test-image-startup.py` | Native startup and child-environment regression execution. |

Paths above are relative to `compiler/native`. Its `src` and `scripts`
subtrees and the three root build files match `fb4c4bc` exactly. Newly imported
upstream documentation and research files remain. The coordinator, semantic
inspection code, benchmarks and historical evidence are preserved.

This is source recovery, not a claim of complete native Rust coverage or a new
universal performance result. Existing reports retain their original source
and configuration scope. Use CI for the repaired commit to assess freshly
executed checks, including any failures or skipped work.

## Source guard

The dependency-free Rust guard checks the complete committed or staged native
tree against the accepted fork tree, then rejects tracked worktree edits and
untracked additions in that scope. It reports the separate upstream identity.
The regression test explicitly rejects a reset that replaces maintained fork
fixes with pristine upstream source.

Build the helper outside the source tree and run it from the repository root:

```sh
rustc --edition=2024 tools/lust-upstream-check.rs -o /absolute/cache/lust-upstream-check
/absolute/cache/lust-upstream-check
```

`--index` checks a staged change before commit. The guard is an integrity check,
not compiler correctness or performance qualification. It does not certify
ignored build output or external dependencies.

## Continuing development

Changes inside `compiler/native` are allowed. Review the diff against both the
upstream base and the current fork. Preserve prior behavior or document the
intended replacement with executable regression evidence. Update the accepted
`tree` pin and this change record in the same commit. The guard's tests exercise
a further reviewed extension, so the recovery pin is not a permanent freeze.

Correctness repairs do not require an unrelated speedup to be retained.
Performance changes need equivalent-workload measurements with source,
compiler, configuration and cache identities, including slower or failed runs.
Do not relabel correctness evidence as a speed result. Keep native acceptance,
rejection, runtime, ownership, panic/drop and warm/cold checks appropriate to
the changed behavior.
