# Lustr

Lustr is being developed to extend the native [Lust compiler](https://github.com/t3dotgg/lust-lang)
to full Rust while retaining its fast frontend, incremental daemon and
development code generation. blueFe is the toolchain integration project.

Existing Lust-supported code must use unchanged upstream Lust. Lustr must supply
the missing Rust language, library and crate support in its own code. Full, unchanged Rust
projects with measured fast compilation are the requirement. Forwarding them to
rustc does not satisfy it. See the [native compiler contract](docs/compiler-set/native-contract.md).

Full native Rust coverage and the Lustr expansion interface are incomplete.
`compiler/native` is the unchanged upstream Lust source, pinned by
[`upstream/lust.pin`](upstream/lust.pin). Lustr-owned expansion belongs outside
that baseline. No Lust divergence is admitted. Any exception must demonstrate
both necessity and faster results under the [upstream preservation policy](docs/compiler-set/upstream-lust.md).
The current root Cargo workspace still builds the separate
compatibility coordinator. That workspace and the published rustc-based previews
are transitional tools, not the completed native compiler.

Build the native compiler with an existing Rust bootstrap toolchain and the
pinned TinyCC assets. Keep generated output outside the source checkout:

```sh
LUST_TCC_HOME=/absolute/tcc CARGO_TARGET_DIR=/absolute/lustr-native-build \
  cargo build --manifest-path compiler/native/Cargo.toml --release --locked
/absolute/lustr-native-build/release/lust check unchanged.rs
/absolute/lustr-native-build/release/lust build unchanged.rs -o application
```

## Native and compatibility package

The [qualified Linux x86_64 package at `595cdc4`](https://github.com/glopyglerky/lustr/actions/runs/42977250507/artifacts/13836362511)
contains native Lust, its matching TinyCC runtime and source libraries, the
Lustr coordinator, and official Rust 1.99.1 for full Rust compatibility.
Download the artifact while signed into GitHub, extract its ZIP, verify
`SHA256SUMS`, and extract the contained tarball. It needs a Linux system compatible
with Ubuntu 24.04, C development headers and a C linker.

```sh
sha256sum -c SHA256SUMS
tar -xzf lustr-native-preview-x86_64-unknown-linux-gnu.tar.gz
LUSTR_ROOT="$PWD/lustr-native-preview-x86_64-unknown-linux-gnu"

# Native Lust, with its original compiler and daemon path.
"$LUSTR_ROOT/bin/lust" build main.rs -o app
./app

# Full Rust compatibility through the bundled Rust compiler.
"$LUSTR_ROOT/bin/lustr" --bluefe-root "$LUSTR_ROOT" --engine rustc build --locked
"$LUSTR_ROOT/bin/lustr" --bluefe-root "$LUSTR_ROOT" --engine rustc test --locked
```

Direct `bin/lust` avoids the optional coordinator's dispatch cost. Native Rust
coverage remains incomplete. The [package evidence](https://github.com/glopyglerky/lustr/actions/runs/42977250507/artifacts/13836655982)
records unchanged compiler bytes, relocation, actual native and compatibility
execution, unsupported native features, and paired cold and warm measurements.
The [package guide](docs/compiler-set/native-package.md) explains selection,
source archives and rebuild instructions. These results do not establish full
native Rust coverage or exact preservation of every upstream speed ratio.

## Earlier compatibility toolchain

For versioned binary bundles, see [BlueFe releases](https://github.com/glopyglerky/blueFe/releases)
and [Lustr releases](https://github.com/glopyglerky/lustr/releases). Use a release's
recorded platform, component manifest and checksums. Keep the extracted toolchain
at a versioned absolute path; ordinary project builds do not rebuild the compiler.

[Preview 2026.11.09-2](https://github.com/glopyglerky/lustr/releases/tag/preview-2026.11.09-2)
is available for Linux x86_64 GNU with GLIBC 2.39 and GLIBCXX_3.4.32. Download all
six assets and verify `SHA256SUMS`. It includes the matched rustc-based BlueFe
compiler, host/Wasm libraries, Clippy/rustfmt and experimental unwind-enabled Cranelift.
Other hosts and Wasm execution are unqualified. The release records coordinator
source `07470b2`; the later default-branch benchmark/docs commit leaves that
coordinator source unchanged. See the
[exact release record](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/releases/preview-2026.11.09-2.md).
It does not include native Lust. Its measurements do not measure native Lust
acceleration or establish full native Lustr coverage.

```sh
lustr --bluefe-root /absolute/toolchain build --workspace
lustr --bluefe-root /absolute/toolchain test --workspace
lustr --bluefe-root /absolute/toolchain build --target wasm32-unknown-unknown
lustr --bluefe-root /absolute/toolchain doctor
```

The assembled BlueFe directory supplies Cargo, rustc, rustdoc and the produced
sysroot. Full Rust currently goes through rustc. A standalone coordinator's
`lustr build` without a root preserves the project's existing Cargo toolchain.
In a marked native preview package, the coordinator selects its bundled
toolchain when no explicit compiler or toolchain selection overrides it.
Arguments, profiles, features,
compiler wrappers and failure status remain effective. Builds install nothing
and failed programs are never retried through another compiler.

An explicit Cranelift engine provides development code generation when a matching,
unwind-enabled backend has passed qualification. WebAssembly builds use
LLVM. A separately pinned backend cannot be loaded into a different BlueFe compiler.
The default forwarding path removes no compiler work by itself.

An experimental, explicit frontend job option applies to one selected Cargo
`rustc` target without replacing project flags or compiler wrappers:

```sh
lustr --bluefe-root /absolute/toolchain --frontend-jobs 4 \
  rustc --profile check -p your-package --lib
```

Select an actual package/target and a bounded CPU budget. More threads can regress
small targets or increase memory. The [argument-preservation proof and executable
checks](docs/compiler-set/proofs/frontend-scheduling.md) state the guarantees and
limits; measured speed claims require fixed-workload evidence.

Building the coordinator itself is optional contributor work:

```sh
cargo build --release --locked
cargo install --path crates/lustr --locked
```

## Source map

| Path | Responsibility |
| --- | --- |
| `compiler/native` | Exact pinned upstream Lust source, including its native compiler, libraries, TinyCC integration and differential fixtures. No Lustr-owned edits are admitted here. |
| `crates/lustr` | Transitional Cargo/rustc coordinator, BlueFe tool selection and compatibility qualification. |
| `upstream/lust.pin` and `tools/lust-upstream-check.rs` | Upstream revision/tree identity and the Rust source-preservation guard. |
| `crates/lustr-semantic` | Optional version-pinned rustc inspection. It exports inspection data, not executable MIR. |
| `crates/lustr-bench` | Standalone paired benchmark runner and correctness gates. It is outside the coordinator workspace. |
| `docs/compiler-set` | Current design, decisions, divergence records, measured evidence and future work. |

The coordinator does not depend on TinyCC or registry packages. `cargo-lustr` and
`cargo-lust` remain command aliases. The native component has a separate Cargo
workspace so its C dependencies do not become prerequisites for installing the compatibility coordinator.

## Working contract and research

Native execution, workspace tests, doctests, invalid-borrow rejection and wasm
linking have dedicated BlueFe gates. Backend qualification also checks both
Rust ABI directions, ownership transfer and panic cleanup. Exact compiler heads,
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
