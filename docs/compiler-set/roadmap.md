# Compiler-set implementation roadmap

This roadmap preserves the full Rust acceleration goal. Each item has an executable acceptance condition. The initial coordinator is a starting point; tasks remain open until their own code and evidence exist.

## Implementation status

The [verification report](verification.md) identifies the exact tested commits and retains the measurements and failures.

| Item | Current status |
| --- | --- |
| 1. Cargo entry point | Implemented. Ordinary Cargo qualification passes on Linux, macOS, Windows, and Rust 1.85. Actual native Lust and pinned Cranelift execution gates pass on Linux x86_64. |
| 2. Backend qualification | Partial. Panic recovery, destructors, full-language fixtures, nested Cargo, and bidirectional LLVM/Cranelift Rust ABI tests pass with the pinned backend. The new mixed gate covers aggregates, generics, trait objects, niches, allocation ownership, exact unwind cleanup order, atomics, TLS, and C ABI callbacks. The broader feature and platform matrix below remains open. |
| 3. Workload measurements | Initial evidence is retained for the small fixture and unchanged upstream Lust. The full-project run passed both 391-test suites and all build/runtime gates; two cold pairs measured 59.54 s with LLVM versus 39.40 s with Cranelift. No-op builds regressed. The broader census and profiling requirements below remain open. |
| 4. Automatic Lust routing | Open. Native Lust selection remains explicit and applies to complete supported targets. |
| 5. Shared semantic interface | Partial. The pinned `lustr-semantic` driver reads checked monomorphized items, symbols, function ABIs, layouts and cleanup edges. Twelve real-compiler integration tests pass. Reports are inspection-only, not executable IR or complete cache keys. Native Lust lowering and host/target separation remain open. |
| 6. Lust generation from full Rust IR | Open. Native Lust does not participate inside Cargo's Rust library graph. |
| 7. Persistent full Rust frontend | Open. The complement still invokes rustc for full-language analysis. |
| 8. Additional compiler workers | No further independent frontend selected. A split requires the measurements and equivalence checks below. |
| 9. Installation and release qualification | Partial. The coordinator installs independently, and the pinned backend has a Linux setup helper. A complete supported release bundle and broader host/target qualification remain open. |

## 1. Cargo entry point

Deliver a dependency-free, independently installable `cargo-lust` binary. Default builds use the selected Cargo toolchain with its exact arguments. Expose explicit Rust, Cranelift, and whole-target Lust paths, and a plan view that explains the decision.

Acceptance: real offline workspaces exercise build scripts, proc macros, async, unsafe/raw pointers, generic associated types, const generics, cross-crate traits, panic recovery, and tests. Invalid Rust remains rejected. Check process arguments, stdout/stderr, exit status, signals, profile selection, runtime arguments, and no automatic command retries. No tcc dependency for installing the coordinator.

## 2. Backend qualification

Pin and test a rustc/Cranelift pair on each advertised platform. Add rejection and runtime probes for panic unwinding and drops, SIMD/intrinsics, inline assembly, TLS, atomics, FFI, allocation across crates, target features, debug information, coverage, sanitizer flags, LTO, and cross compilation. Run ordinary Cargo and candidate builds from the same source and configuration.

Acceptance: full results name each supported configuration, unsupported combination, and remaining gap. A backend is selected automatically only within its verified contract. A runtime failure is never retried as a compiler fallback. Unsupported cases retain a complete Cargo path.

## 3. Workload census and measurements

Inventory representative small CLIs, large libraries, macro-heavy services, async runtimes, embedded/no_std crates, and workspaces with native dependencies. Begin with small reproducible fixtures, then real projects. Use exact upstream revisions and record modifications; prefer unchanged projects.

Acceptance: raw, interleaved cold/warm/body/signature/type/macro/rejection samples, medians and p95, peak memory, artifact sizes, and compile-plus-test times. Record hardware, jobs, toolchain/backend identities, profile, features, and panic strategy. Include regressions. Use profiles to choose the next compiler work.

## 4. Automatic complete-target Lust routing

Design and implement a versioned qualification record tied to the selected Cargo unit and its complete transitive closure. Identify adapter substitutions, excluded features, build-script inputs, generated source, target/profile assumptions, and unsafe/proc-macro dependencies. Reuse Lust's existing session keys and read tracking where their contract applies.

Acceptance: supported complete targets use Lust automatically without changing requested behavior; unqualified requests take Cargo. A workspace with several targets builds every requested target. Changing any tracked input invalidates the record. Fuzz edits against both cold Lust and rustc. Do not pretend Lust libraries have Rust metadata.

## 5. Shared full Rust semantic interface

Prototype a version-pinned rustc driver/backend integration that exposes checked monomorphized bodies and authoritative ABI/layout information. Keep metadata, symbol identity, generic instantiation ownership, and Cargo artifact production with rustc. Specify drop/unwind edges, intrinsics, static initialization, atomics, TLS, and diagnostics before handing work to a new backend.

Acceptance: compile and link both directions across the real Rust crate interface, including generic calls, trait objects, enums with niches, returned aggregates, unwinding with destructors, proc-macro generated code, and host/target separation. Tests must fail when compiler/ABI versions mismatch. A C ABI demonstration alone does not close this item.

## 6. Lust code generation behind the shared interface

Adapt the useful parts of Lust's C generation, runtime, and fast compilation to input from item 5. Preserve the original Lust frontend as its own fast family. Unsupported operations fall back before publishing output and before executing user code. Define whether mixed backend code uses one allocator/runtime and who owns every generic instance and drop function.

Acceptance: differential build/run/reject tests across mixed codegen units, correct metadata and linker outputs, stable debug locations, and real compile-plus-test improvement. No duplicate semantic frontend work on the accelerated route.

## 7. Persistent full Rust frontend

Use measurements from item 3 to select retained semantic computations. Specify lifetimes and ownership of retained state. Track positive and negative reads of names, impl candidates, signatures, constants, macro expansions, build-script observations, and external metadata. Start with one edit class and expand only with evidence.

Acceptance: randomized long edit sequences match fresh-process rustc acceptance, diagnostics, and executable behavior. Type, trait, feature, target, generated-source, and toolchain changes cannot reuse stale answers. Bound retained memory and recover from compiler crashes without publishing a successful result.

## 8. Decide whether more compiler workers help

Compare a single full Rust complement with parallel function/codegen-unit workers. Only consider another frontend if a measured semantic partition can avoid duplicated analysis. Keep worker communication explicit and versioned.

Acceptance: same inputs and results, meaningful end-to-end improvement including serialization/linking, no jobserver oversubscription, bounded memory, and stable p95. Remove a split that adds cost. Worker count follows the workload and available resources.

## 9. Installation and release qualification

Package the coordinator, compatible Lust runtime/tcc assets, and clearly identified complementary backend requirements. Preserve upstream licenses and isolate upstream sync changes from fork code. Keep release profiles and program runtime performance in the test matrix.

Acceptance: installation from a clean supported host, offline repeat builds, upgrades without stale cache reuse, documented rollback to ordinary Cargo, and end-to-end tests on every advertised platform. Do not call an installer universal merely because its coordinator compiles on several operating systems.

## Closure rule

Do not close the full project objective while full-language semantics come only from a compatibility fallback with no demonstrated acceleration. Do not close artifact interoperability, full frontend persistence, or automatic selection when only plans or mocked tests exist. Keep the current implementation, evidence, and remaining work visible in the README and verification report.
