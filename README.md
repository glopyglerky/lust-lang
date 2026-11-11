# Lustr

Lustr is being developed to extend the native [Lust compiler](https://github.com/t3dotgg/lust-lang)
to full Rust while retaining its fast frontend, incremental daemon and
development code generation. BlueFe holds stock Rust source and must use Lustr
as its only Rust compiler.

Existing Lust-supported code must use unchanged upstream Lust. Lustr must supply
the missing Rust language, library and crate support in its own code. Full, unchanged Rust
projects with measured fast compilation are the requirement. Forwarding them to
rustc does not satisfy it. See the [native compiler contract](docs/compiler-set/native-contract.md).

Full native Rust coverage remains incomplete. The experimental
[`lustr-native`](crates/lustr-native/README.md) entry point now reuses the native
pipeline in process and implements a bounded Cargo interface for binaries and
independently compiled native libraries with safe scalar function exports.
`compiler/native` is the unchanged upstream Lust source, pinned by
[`upstream/lust.pin`](upstream/lust.pin). Lustr-owned expansion belongs outside
that baseline. No Lust divergence is admitted. Any exception must demonstrate
both necessity and faster results under the [upstream preservation policy](docs/compiler-set/upstream-lust.md).
Cargo owns toolchain and compiler selection. The native package has no additional
coordinator or engine-selection process. The earlier coordinator source and
rustc-based previews remain historical compatibility work; they do not implement
the missing native compiler support.

Build the native compiler with an existing Rust bootstrap toolchain and the
pinned TinyCC assets. Keep generated output outside the source checkout:

```sh
LUST_TCC_HOME=/absolute/tcc CARGO_TARGET_DIR=/absolute/lustr-native-build \
  cargo build --manifest-path compiler/native/Cargo.toml --release --locked
/absolute/lustr-native-build/release/lust check unchanged.rs
/absolute/lustr-native-build/release/lust build unchanged.rs -o application
```

## Native compiler package

The current package recipe supplies unchanged `lust` and `lust-core`, the real
`lustr-native` compiler, Cargo, TinyCC and native runtime/source assets. The
native product excludes stock rustc, rustdoc, a stock Rust sysroot and coordinator
executables. Rustc is used separately to bootstrap Lustr and compare results.

Lustr now has an owned native library emitter and metadata importer. A library
contains real relocatable objects plus typed export metadata. A consumer can
link that library after its producer source is hidden. The initial interface
supports safe primitive function signatures; its [entry-point guide](crates/lustr-native/README.md)
records the remaining language, module, generic and crate limitations.

Cargo selects the compiler directly:

```sh
LUSTR_ROOT=/absolute/path/to/lustr-native-preview-x86_64-unknown-linux-gnu
RUSTC="$LUSTR_ROOT/bin/lustr-native" "$LUSTR_ROOT/bin/cargo" build --locked
```

Full native Rust support and BlueFe compilation remain unfinished. The required
library fixture currently exposes unsupported unsafe Rust. Package qualification
retains that failure and does not upload an installable archive while its gate
is unsuccessful. See the [native package guide](docs/compiler-set/native-package.md)
for actual scope, source/runtime identities and qualification rules.

The unchanged upstream direct compiler also has a reproduced external-input
cache defect involving whitespace/comments before macro `!`. Its historical
reproducer and workaround remain recorded. The owned Cargo path does not use
that whole-request cache. Native Cargo daemon reuse and exact preservation of
all Lust speed gains remain unqualified.

## Earlier compatibility work

The [2026.11.09-2 preview](https://github.com/glopyglerky/lustr/releases/tag/preview-2026.11.09-2)
contained a rustc-based toolchain and a coordinator, with no native Lust compiler.
Its [release record](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/releases/preview-2026.11.09-2.md)
and earlier compiler experiments remain available as historical evidence.
The coordinator is not part of the current package or the planned native build
path. Native implementation must satisfy the existing Cargo/rustc contract
without adding another dispatch layer.

## Source map

| Path | Responsibility |
| --- | --- |
| `compiler/native` | Exact pinned upstream Lust source, including its native compiler, libraries, TinyCC integration and differential fixtures. No Lustr-owned edits are admitted here. |
| `crates/lustr-native` | In-process native compiler, Cargo argument handling, native object/metadata emission, typed library imports and real dependency records. |
| `crates/lustr` | Historical compatibility coordinator and reusable Rust fixtures. Its coordinator is not built or shipped in the native package. |
| `upstream/lust.pin` and `tools/lust-upstream-check.rs` | Upstream revision/tree identity and the Rust source-preservation guard. |
| `crates/lustr-semantic` | Optional version-pinned rustc inspection. It exports inspection data, not executable MIR. |
| `crates/lustr-bench` | Standalone paired benchmark runner and correctness gates. It is outside the coordinator workspace. |
| `docs/compiler-set` | Current design, decisions, divergence records, measured evidence and future work. |

The native component and its new Cargo entry point have separate Cargo
workspaces. Packaging builds both native executables and the matching runtime;
the root coordinator workspace is not a package build
dependency. Earlier coordinator aliases remain in source history and research
code, outside the native product path.

## Working contract and research

Earlier BlueFe compatibility gates exercise execution, workspace tests,
doctests, invalid-borrow rejection, wasm linking and backend ABI behavior. Their
results describe the historical rustc-based path. The current Lustr-only gate
must establish native compiler coverage separately. Exact compiler heads,
source fingerprints, backend hashes and unsupported targets belong with results.

See the [integration and performance model](docs/compiler-set/bluefe-performance.md),
[verification record](docs/compiler-set/verification.md),
[design](docs/compiler-set/design.md), and [roadmap](docs/compiler-set/roadmap.md).
Performance claims are workload-specific. No-op, edit, rejection and executed-test
costs count alongside cold compilation. Slower candidates remain in the evidence.

The [measurement protocol](docs/compiler-set/benchmark-method.md) records exact
tools, dependency selection, compiler commands, correctness gates and every
paired sample. Its [mathematical analysis](docs/compiler-set/proofs/benchmark-analysis.md)
proves estimator invariants and conditional statistical properties. The
[metadata comparison proof](docs/compiler-set/proofs/metadata-compatibility.md)
states the one supported absent/null schema equivalence and its limits. These proofs
do not establish compiler equivalence or an unmeasured speedup. The runner's tiny
fixture qualifies the measurement machinery; public-repository campaigns remain
separate evidence.

The runner also supports [optional Linux CPU and peak-RSS observations](docs/compiler-set/proofs/benchmark-resources.md).
Fresh collectors preserve payload outcomes and avoid cumulative-peak leakage.
RSS means the largest waited process, so it cannot establish aggregate build
memory. The historical shared-v2 dataset remains wall-time-only. The matched pilot below
records CPU and scoped RSS; the collector itself removes no compiler work.

The [matched CGU16/ThinLTO pilot](docs/compiler-set/measurements/2026-11-09-cgu16-thinlto-v3/README.md)
completed 108 timed root checks plus nine excluded warmups. The optimized stage2
package used 17.9% less wall time than the matched no-PGO stage2 control across
the three fixed cases, with conditional candidate/control interval [0.7995, 0.8401].
It still used more time than stock Rust 1.99.1 on every workload. That secondary
comparison has unequal compiler/Cargo versions. The report includes every command
record, CPU and scoped RSS, regressions, independent review, a Rust replay and
[mathematical arguments](docs/compiler-set/proofs/cgu16-thinlto.md). These are
metadata-only checks, not full-build or application-speed results.

The [2026-11-09 public-repository dataset](docs/compiler-set/measurements/2026-11-09-shared-v2/README.md)
contains 72 timed root checks and nine excluded warmups. Explicit four-thread
BlueFe beat stock on t3code-ls, with a paired ratio of 1.268, but regressed by
point estimate on gitr and bluefin. One-thread BlueFe regressed on all three.
Same-compiler N1/N4 ratios were 2.345, 1.342 and 1.802. Unequal toolchain versions,
shared-host effects and fixed middle position for N1 limit interpretation;
runtime gates did not apply the timed frontend options. These are metadata-only
root checks, not default whole-build results. Full samples, conditional intervals,
regressions, scoped proofs and a Rust statistics replay command accompany the data.

For future campaigns, the runner now balances lane execution positions in cyclic
blocks. Its [position theorem and checks](docs/compiler-set/proofs/order-balance.md)
prove the count bound, without claiming to remove cache effects or establish a
speedup. Published historical measurements keep their original order and limits.

Backend campaigns can use the [routed test gate](docs/compiler-set/proofs/routed-test-gates.md)
with an explicit Lustr toolchain and engine. It checks the core tool selectors;
the native fixture observes LLVM and matching Cranelift compiler commands and
tests panic cleanup. Project-specific gates and matching backend artifacts are
still required before reporting performance.

## Fork ownership

Lustr owns this layout, its entry points and its documentation. Required inherited
compiler code is maintained as part of the native component. Upstream research
results describe their original revisions and workloads; they are not claims
about Lustr's current toolchain.

[Changes from upstream and retained dependencies](docs/compiler-set/divergences.md)
records why inherited components remain and how updates are qualified. The source
base is [Lust](https://github.com/t3dotgg/lust-lang) at
`240e08d91d4aed0299639f6f11fd522912b248f9`. Its original README and history remain
available there and in this repository's Git history. Licensing is preserved.

This is pre-alpha software. The coordinator forbids unsafe Rust in its own code.
The current native compiler, TinyCC, rustc, LLVM and platform dependencies still
require review and eventual migration before a whole-toolchain memory-safety
claim is justified.
