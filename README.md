# Lustr

Lustr is a compiler coordinator for BlueFe and ordinary Cargo projects. Its
current priority is a working development toolchain with measured improvements.
Full native Rust compilation and the migration to memory-safe Rust are future
work, developed without breaking the usable compiler path.

The default Cargo workspace builds only Lustr's dependency-free Rust coordinator.
The native subset compiler and the experimental semantic driver are separate
components with their own requirements.

For versioned binary bundles, see [BlueFe releases](https://github.com/glopyglerky/blueFe/releases)
and [Lustr releases](https://github.com/glopyglerky/lustr/releases). Use a release's
recorded platform, component manifest and checksums. Keep the extracted toolchain
at a versioned absolute path; ordinary project builds do not rebuild the compiler.

[Preview 2026.11.09-1](https://github.com/glopyglerky/lustr/releases/tag/preview-2026.11.09-1)
is available for Linux x86_64 GNU with GLIBC 2.39 and GLIBCXX_3.4.32. Download all
six assets and verify `SHA256SUMS`. It includes the matched BlueFe compiler,
native/Wasm libraries, Clippy/rustfmt and experimental unwind-enabled Cranelift.
Other hosts and Wasm execution are unqualified. The release records coordinator
source `07470b2`; the later default-branch benchmark/docs commit leaves that
coordinator source unchanged. See the
[exact release record](https://github.com/glopyglerky/blueFe/blob/main/docs/bluefe/releases/preview-2026.11.09-1.md).

```sh
lustr --bluefe-root /absolute/toolchain build --workspace
lustr --bluefe-root /absolute/toolchain test --workspace
lustr --bluefe-root /absolute/toolchain build --target wasm32-unknown-unknown
lustr --bluefe-root /absolute/toolchain doctor
```

The assembled BlueFe directory supplies Cargo, rustc, rustdoc and the produced
sysroot. Full Rust currently goes through rustc. `lustr build` without a root
preserves the project's existing Cargo toolchain. Arguments, profiles, features,
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
| `crates/lustr` | Primary command, Cargo integration, BlueFe selection, qualification and performance tools. |
| `compiler/native` | Lustr's native subset compiler, derived from Lust. TinyCC, its fixtures and native build helpers remain here. The `lust` and `lust-core` executable names preserve compatibility with that compiler's tools. |
| `crates/lustr-semantic` | Optional version-pinned rustc inspection. It exports inspection data, not executable MIR. |
| `crates/lustr-bench` | Standalone paired benchmark runner and correctness gates. It is outside the coordinator workspace. |
| `docs/compiler-set` | Current design, decisions, divergence records, measured evidence and future work. |

The coordinator does not depend on TinyCC or registry packages. `cargo-lustr` and
`cargo-lust` remain command aliases. The native component has a separate Cargo
workspace so its C dependencies do not become prerequisites for installing Lustr.

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
