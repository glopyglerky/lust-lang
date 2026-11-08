# Coordinated Rust compilers

Status: initial implementation and development contract, 2026-11-08.

## Goal

An unchanged Rust project should be able to select this compiler set with one command. The set should use Lust where its narrow frontend and warm daemon are correct and useful, and a complementary compiler for the rest of Rust. The goal includes complete Cargo workspaces, their dependencies, build scripts, proc macros, tests, and release builds. It does not require application authors to rewrite their project into Lust's subset.

Compatibility and acceleration are separate properties. The initial Cargo path obtains compatibility by using the project's existing Rust toolchain. It does not replace rustc's frontend or make that path faster by itself. Cranelift is an explicit development backend experiment. Existing Lust remains the fast subset compiler. Automatic coordination between those compiler families is further work with explicit acceptance gates.

## Decision

Keep two compiler families and three compilation paths:

| Path | Semantic frontend | Code generation | Intended use |
| --- | --- | --- | --- |
| Lust | Upstream Lust subset | Lust C output and tcc | Qualified complete targets and warm edits |
| Full Rust development | rustc | Cranelift | Full-language development builds on verified backend configurations |
| Full Rust compatibility | rustc | Project-selected backend, normally LLVM | Compatibility, release optimization, unsupported fast paths |

The full Rust development and compatibility paths share the same semantic frontend. They are not two new independent implementations of the Rust language. This is deliberate. Reusing the compiler's existing resolver, trait solver, borrow checker, const evaluator, layout rules, monomorphization, and metadata gives the complement a viable starting point.

Do not create separate unsafe, async, generic, or proc-macro compilers. Those features overlap in the same functions and types. For example, a generic async function may hold a borrow across a suspension point, call an unsafe operation, and use a trait implementation emitted by a proc macro. Dividing that source by keywords would require the compilers to reconstruct the same global semantic decisions. It offers no demonstrated speed benefit.

Further splitting can make sense *after* semantic analysis. Function or codegen-unit workers can receive immutable, checked input and compile in parallel. A persistent frontend can retain query results across edits. Both designs need profiling and a precise interface before implementation.

## Upstream principles retained

The inspected upstream and fork start at [`240e08d91d4aed0299639f6f11fd522912b248f9`](https://github.com/t3dotgg/lust-lang/commit/240e08d91d4aed0299639f6f11fd522912b248f9).

1. An accepted Lust program is valid Rust and must match the corresponding rustc program. Unsupported input is rejected explicitly.
2. The fast development path avoids expensive optimizing code generation. Release builds preserve the project's normal optimization recipe.
3. Retained compiler state and precise dependency reads reduce work after an edit. A body edit should not require checking unrelated bodies when the compiler can establish that they are unchanged.
4. Uncertain incremental answers take a more complete path. Differential tests compare warm and cold results and use rustc as an oracle.
5. Build time and generated-program runtime are both measured. A faster compiler can lose overall when its test binaries run more slowly.

These principles come from the [upstream README](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/README.md), the executable [driver](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver), and its [plan](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/docs/PLAN.md). The plan contains historical estimates; they are not measurements of this fork.

## Why arbitrary Rust libraries cannot be mixed today

The current Lust driver explicitly rejects building a library target because it has no rlib format. It can check a library and compile source dependencies within a selected binary's closure. That is different from producing a library that Cargo can supply to a rustc-dependent crate. See [`src/driver/compile.rs`](../../src/driver/compile.rs).

Rust libraries carry more than object code. Their metadata contains compiler identity, crate identities, exported types and traits, macros, and sometimes MIR. Cargo can use emitted metadata to start dependent compilation before object generation finishes. The metadata format and Rust ABI cannot be replaced by a C object file and an empty metadata file. See the [Rust compiler metadata guide](https://rustc-dev-guide.rust-lang.org/backend/libs-and-metadata.html).

Type layout and function-call ABI are also separate obligations. Matching a struct's size does not establish ABI compatibility. Generic instances, trait objects, enum niches, closure environments, drop glue, unwinding, thread-local storage, and allocator ownership need agreement. The [Rust layout reference](https://doc.rust-lang.org/reference/type-layout.html) explicitly limits its guarantees.

The initial coordinator therefore has two real interfaces:

- A Cargo invocation remains a Cargo invocation. Cargo owns resolution, features, target units, build scripts, proc macros, metadata, artifact freshness, linking, tests, and documentation.
- An explicit Lust invocation remains a complete Lust target invocation. Lust owns its frontend, adapters, runtime, cache, and daemon. The coordinator does not tell Cargo that this output is a Rust library.

The coordinator does not parse application source or recreate Cargo's dependency graph merely to choose a compiler.

## Cranelift policy

Use the [official Cargo profile backend mechanism](https://doc.rust-lang.org/cargo/reference/unstable.html#codegen-backend). Keep the selected Rust toolchain. Do not install a nightly, use `RUSTC_BOOTSTRAP`, change the user's default toolchain, or rewrite project manifests during a build.

Supply the dev/test backend through command-scoped `--config` arguments. Setting `CARGO_PROFILE_*_CODEGEN_BACKEND` in the environment also passes those unstable settings to applications, tests, and their child processes. A nested ordinary Cargo invocation can then fail because it did not request the unstable backend feature. The configured compiler choice belongs to this Cargo command; inherited user settings remain the user's settings. Plans describe that requested command, while runtime qualification and benchmark checks inspect actual per-crate compiler commands.

Cranelift is opt-in. Its [current upstream README](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md) lists incomplete `std::arch` SIMD support and experimental panic unwinding, with narrower platform coverage than rustc. Its standalone distribution also defaults to aborting panics. A near-drop-in compiler is not a universal semantic guarantee.

The first real backend gate found a concrete incompatibility. The rustup preview on nightly-2026-11-07 failed the panic-recovery fixture. Upstream's [`unwinding` feature](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/Cargo.toml) is disabled by default. Without it, the [`catch_unwind` intrinsic](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/src/intrinsics/mod.rs) calls the closure directly, and generated cleanup behavior is incomplete. Keeping Cargo's panic flag alone cannot fix that.

The complementary backend is therefore built explicitly from pinned source with unwind support, using its matching nightly-2026-11-05 compiler. `LUST_CRANELIFT_BACKEND` identifies the resulting absolute library path. The coordinator refuses the stock named default for this path. The setup helper installs the pinned build toolchain only when invoked explicitly; normal compilation installs nothing. Full runtime qualification remains mandatory, and this backend is still not a universal replacement for LLVM.

The coordinator must not change panic strategy, suppress overflow checks, remove features, reduce debug information, or change optimization levels to improve benchmark results. Release and custom profiles retain Cargo's settings. A failed compilation, program, or test returns its failure; the coordinator does not repeat the command under another engine.

`cargo check` usually avoids ordinary native code generation, so changing the backend does not address most of its work. Upstream Lust's own plan records a ts-rust case where Cranelift took 53.8 seconds against 43.0 seconds for rustc's normal backend. That is a useful counterexample to a universal speed claim, not a prediction about other workloads.

## Automatic target qualification

Before selecting Lust automatically, a qualification record must identify the exact requested target and dependency closure. It must include:

- Compiler, runtime, adapter, and toolchain identities.
- Target triple, edition, features, profile, panic strategy, overflow checks, and debug settings.
- Manifest and lockfile identities, source dependencies, registry dependencies, proc macros, build scripts, and their observed inputs and outputs.
- Any substitution of a dependency by an adapter and the evidence for its required behavior.
- Acceptance and rejection parity, runtime output and exit status, file effects, test results, and panic/drop behavior.
- Cold, warm, edit, and compile-plus-test timings with the same requested configuration.

A successful Lust compile is insufficient. A fallback on rejection would miss incorrect acceptance, and could silently use reduced features or adapters with different behavior. Qualification must not execute an application's `run` command as a hidden probe. Trusted build-script and proc-macro execution stays explicit through the build process.

Qualification is configuration-specific. A changed feature, adapter, build script, compiler, target, dependency, or semantic assumption invalidates it. A certificate covering yesterday's complete source snapshot does not prove tomorrow's arbitrary edit correct. Warm invalidation rules must be separately tested.

## Future shared semantic interface

The most promising route to coordinated code generation inside arbitrary crates is a rustc-backed, versioned semantic interface. Start with a fixed compiler revision and pass checked, monomorphized bodies with authoritative layouts, calling conventions, symbols, drop/unwind operations, intrinsics, and linkage requirements to the chosen backend.

Initially rustc must remain responsible for metadata publication and Cargo's contract. A C ABI experiment can test a narrow adapter, but cannot establish general Rust ABI compatibility. Moving Lust's code generator behind this interface requires an actual mapping from rustc MIR to Lust's code-generation expectations. That mapping does not exist in this commit.

A persistent full Rust frontend is a separate compiler project. It must account for macro expansion, coherence, negative lookups, trait candidate changes, build-script generated code, const evaluation, and cross-crate metadata. Reusing a rustc query context beyond its valid lifetime is not a daemon implementation. Retained work needs explicit ownership and invalidation rules, plus cold-session equivalence tests.

## Performance decisions

Add a specialized compiler only when it removes enough measured work to pay for dispatch, serialization, duplicate analysis, cache invalidation, linking, memory use, and slower generated programs. Compare one complement against further partitioning on the same workload. The default design is one full-language complement because there is no evidence yet that several independent frontends would help.

For a baseline compilation time `T`, if fraction `p` is accelerated by factor `s`, the ideal time before coordination overhead is `T * ((1 - p) + p / s)`. Even infinitely fast code generation cannot remove frontend time. Real benchmarks must include coordination overhead and the runtime of executed tests.

Record cold compilation, fresh-process cache reuse, no-op builds, body edits, signature edits, type/trait changes, macro edits, invalid edits, linking, and compile-plus-test time separately. Measure medians and tails, interleave baseline and candidate, retain raw data, and disclose target hardware and compiler identities. Do not describe an eight-line example as proof for a large workspace.

## Completion criteria

This project is complete only when ordinary full Rust projects can use the set without source rewrites, the selected fast paths are semantically qualified, artifacts interoperate where promised, and equivalent-workload measurements show an improvement on representative projects. The coordinator, a few passing fixtures, or integration with an existing fast backend alone does not meet that definition.

The implementation sequence and its gates are in [the roadmap](roadmap.md).
