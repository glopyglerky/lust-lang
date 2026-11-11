# Native library implementation and BlueFe build evidence

Date: 2026-11-11. The native compiler now independently compiles and links the
bounded safe-library cases below. **Full native Rust support is incomplete, the
required unsafe library gate fails, and BlueFe has not been built by Lustr.**
These local results do not qualify an installable full-replacement package or
establish preservation of Lust's compilation speed.

## Published implementation

- Lustr: [`9fd796650b935db4a25de58263a7035e946d14d7`](https://github.com/glopyglerky/lustr/commit/9fd796650b935db4a25de58263a7035e946d14d7), [draft PR 4](https://github.com/glopyglerky/lustr/pull/4).
- BlueFe: [`94fd9940bf37268ca6d7fce95841eb1fff3c0c78`](https://github.com/glopyglerky/blueFe/commit/94fd9940bf37268ca6d7fce95841eb1fff3c0c78), [draft PR 1](https://github.com/glopyglerky/blueFe/pull/1). The initial integration is commit `80112a3`; `94fd994` adds the missing locked dependency fetch before the offline compiler build.

The retained final compiler source files and Lust pin were compared byte for
byte with the published Lustr commit. The retained BlueFe runner was compared
with the published BlueFe commit. [index.json](index.json) records that mapping
and hashes the evidence files. The local builds happened before these commits
were created; this source comparison is not a substitute for an optimized CI
build receipt.

Lust's protected tree remains `0ab1b09815802894e6078de0af59273cbdf46cb0`, pinned to
upstream `240e08d91d4aed0299639f6f11fd522912b248f9`. BlueFe's protected Rust source
remains pinned to `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`. Compiler extensions
and build integration live outside those protected scopes.

## Final observed results

| Gate | Result | Evidence |
| --- | --- | --- |
| Safe native libraries | All **81 commands** completed with their expected status: 54 successful commands, 9 semantic rejections, 14 unsupported/corruption rejections and 4 expected exit-101 results. No timeouts. | [Safe report](scenarios/candidate-v4-safe-no-trace/report.json), [exact runner](scenarios/candidate-v4-safe-no-trace/runner-source.py), [fixed input fixture](scenarios/candidate-v4-safe-no-trace/initial-fixture) |
| Full required native library fixture | **Failed.** The original parser rejects its unsafe producer with `L0404`; later checks remain unreached. | [Required report](scenarios/candidate-v4-required-no-trace/report.json), [unchanged required fixture](scenarios/candidate-v4-required-no-trace/initial-fixture) |
| BlueFe compiled with final native Lustr | **Failed.** Unchanged `x.py` reaches Cargo's actual compiler query; Lustr returns exit 2 for unsupported `-Zallow-features=`. | [Build log](bluefe-v4/build.log), [configuration](bluefe-v4/bootstrap.toml), [report](bluefe-v4/qualification.json), [actual runner](bluefe-v4/build-runner-source.py) |
| Local execution tracing | Unavailable. An earlier strict attempt failed because the environment denied `ptrace`, before any compiler ran. Later local runs explicitly request diagnostic mode and remain unqualified. | [Infrastructure log](trace-infrastructure/build.log), [strict attempt record](trace-infrastructure/qualification.json) |

The safe suite exercises real Cargo compilation of two versions of the same
crate under different aliases. It inspects actual native ELF object symbols and
typed metadata, hides producer source, and compiles unchanged consumers using
their `--extern` artifacts. It covers scalar ABI boundaries, unit values,
function pointers, cfg behavior, visibility/type/arity/result errors, producer
overflow locations, malformed and foreign archives, unsupported public
contracts, and Cargo freshness after body, signature and artifact edits. A root
export calling a private-module helper remains a positive control.

The safe suite is supplemental. It always leaves `qualified=false` and
`full_required_suite_passed=false`; it cannot replace the unsafe requirements
or make a complete compiler claim. A nonzero exit in its expected-rejection
cases is a passing assertion, not a successful compilation.

The [small contract-check record](small-contract-checks/results.json) retains
three passing artifact codec tests, three passing BlueFe bootstrap/trace tests
and the two passing source-preservation guards.

## Compiler identity and build scope

The final diagnostic compiler has SHA-256:

```text
704b9c9fa348313229581bb458f028f15861b6368ede24f3600648c8fa5268fd
```

Its native build identity is `774f1df0f45c7d7b`. The
[V4 source and build record](compiler-builds/v4-source.json) retains all source
hashes, actual compiler argv and binary identity. The
[observed Cargo build log](compiler-builds/build-v4.log) and
[exact final compiler source](compiler-source-v4) are included.

The observed build uses the pinned Rust 1.99.1 bootstrap, optimization level 1,
16 code-generation units, no embedded bitcode, an owned incremental directory
and stripped debug information. The environment requests LTO disabled. This is
a correctness diagnostic build; the package requires its separate optimized
gate-profile receipt. Recorded command durations are not performance results.

## Regressions found and corrected

The earlier failures remain in this directory:

| Observation | Correction or remaining limit |
| --- | --- |
| [Baseline Cargo cannot produce library targets](scenarios/baseline-f1-no-trace/report.json). | Implemented actual native library objects and typed imports. |
| [V1 accepts an unrepresented `must_use` export](scenarios/safe-supplement-v1/report.json). | Reject export attributes whose caller contract is not represented. |
| [V2 harness assumes the wrong metadata identity string](scenarios/candidate-v2-safe-no-trace/report.json). | Corrected the assertion to bind the documented compiler/runtime identities to the native build hash. This was a test-runner error. |
| [V2 returns exit 2 for a semantic type error](scenarios/candidate-v2-safe-no-trace-2/report.json). | Typed error status now preserves exit 1 for real semantic diagnostics, without adding a duplicate consumer checking pass. |
| [V3 passes the earlier 69-command safe scope](scenarios/candidate-v3-safe-no-trace/report.json), but [accepts three deprecated namespace/crate contracts](export-contract-v3/report.json). | Require original root public function declarations and reject public module/re-export and crate inner-attribute contracts. V4 adds all three rejection cases and retains the private-module positive control. |
| [Required unsafe support still fails in V4](scenarios/candidate-v4-required-no-trace/report.json). | Remains an implementation requirement. The required fixture was not replaced or weakened. |

V2 and V3 compiler source snapshots are retained in
[compiler-source-v2](compiler-source-v2) and [compiler-source-v3](compiler-source-v3).
The index distinguishes complete retained runner/input snapshots from earlier
records recovered by matching their originally recorded fixture hashes. It does
not claim unavailable original runners were retained.

## Required CI and unresolved compiler work

The source commits launched [Lustr package CI](https://github.com/glopyglerky/lustr/actions/runs/43004480716)
and [BlueFe's corrected Lustr-only build](https://github.com/glopyglerky/blueFe/actions/runs/43005192822).
Their own artifacts bind the exact optimized binaries and process observations.
Local diagnostic success is not substituted for those results. The native
package workflow requires the full library gate and withholds archive upload
after a failure. The BlueFe source-preservation run
[passed](https://github.com/glopyglerky/blueFe/actions/runs/43004612202); its
inherited stock-compiler matrix was
[skipped](https://github.com/glopyglerky/blueFe/actions/runs/43004612248) under the
explicit upstream-repository condition.

The [first BlueFe CI attempt](https://github.com/glopyglerky/blueFe/actions/runs/43004612143)
failed before compiling Lustr because its fresh Cargo cache lacked
`libmimalloc-sys`. Its [build log](bluefe-ci-first-attempt/lustr-build.cargo.log)
is retained. The workflow now fetches the locked native dependencies first and
keeps the actual recorded compiler build offline. This initial CI setup failure
is separate from the native `-Zallow-features=` failure observed locally.

Native unsafe semantics, general public types/traits/generics, references and
ownership/drop metadata, transitive libraries, cross-crate macros, procedural
macros, metadata-only compilation, full standard-library integration and many
compiler-protocol requests remain unfinished. New-path daemon/cache behavior
and equivalent-workload speed also require implementation and measurement.
The current artifact format is Lustr-native; interoperability with rustc's
metadata or Rust ABI is not claimed.

The compiler/library build source, test sources and logs are retained here.
Build binaries, objects, rlib/rmeta artifacts, targets and caches are omitted;
their available identities remain in the raw reports. The complete native
replacement remains unfinished.
