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
