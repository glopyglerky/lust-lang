# Native Lustr compiler design

Status: native compiler requirement, following the user's clarification.

## Goal

Keep upstream Lust unchanged and supply full Rust support through Lustr-owned native implementation while retaining and expanding Lust's fast compilation model. Every construct Lust already supports must use Lust. Lustr supplies the missing language, library and crate support through qualified interfaces. blueFe delivers the combined compiler. This includes Cargo workspaces, dependencies, build scripts, proc macros, tests and release builds. Application rewrites into the subset do not satisfy the goal.

The [native compiler contract](native-contract.md) and [upstream preservation policy](upstream-lust.md) are authoritative. No Lust divergence is admitted without demonstrated necessity and faster results. The existing Cargo coordinator obtains compatibility through rustc and does not replace its frontend. Cranelift and rustc build tuning are compatibility research. They do not implement full native Lustr coverage, and their speed measurements cannot be transferred to the native compiler.

## Decision

The required execution paths are:

| Path | Semantic frontend | Code generation | Intended use |
| --- | --- | --- | --- |
| Existing Lust support | Native Lust frontend | Native C output and TinyCC | Retain accepted behavior and the fast edit path |
| Lustr expansions | Lustr-owned native support outside the upstream baseline | Qualified native generation with the required Rust behavior | Add missing Rust features without rewriting applications or hiding Lust patches |
| Explicit compatibility tools | rustc | Project-selected backend | Bootstrap, differential oracle and transitional compatibility; excluded from native coverage |

Lustr's extension interface remains unimplemented. It must preserve resolution, trait selection, borrow checking, const evaluation, layout, monomorphization and incremental invalidation as coverage grows. Mixed programs containing existing Lust features and Lustr expansions must compile and execute natively. The current native coverage remains incomplete; routing a program to rustc does not close a native implementation gap.

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

The current Lust driver explicitly rejects building a library target because it has no rlib format. It can check a library and compile source dependencies within a selected binary's closure. That is different from producing a library that Cargo can supply to a rustc-dependent crate. See [`src/driver/compile.rs`](../../compiler/native/src/driver/compile.rs).

Rust libraries carry more than object code. Their metadata contains compiler identity, crate identities, exported types and traits, macros, and sometimes MIR. Cargo can use emitted metadata to start dependent compilation before object generation finishes. The metadata format and Rust ABI cannot be replaced by a C object file and an empty metadata file. See the [Rust compiler metadata guide](https://rustc-dev-guide.rust-lang.org/backend/libs-and-metadata.html).

Type layout and function-call ABI are also separate obligations. Matching a struct's size does not establish ABI compatibility. Generic instances, trait objects, enum niches, closure environments, drop glue, unwinding, thread-local storage, and allocator ownership need agreement. The [Rust layout reference](https://doc.rust-lang.org/reference/type-layout.html) explicitly limits its guarantees.

The existing compatibility coordinator has two real interfaces:

- A Cargo invocation remains a Cargo invocation. Cargo owns resolution, features, target units, build scripts, proc macros, metadata, artifact freshness, linking, tests, and documentation.
- An explicit Lust invocation remains a complete Lust target invocation. Lust owns its frontend, adapters, runtime, cache, and daemon. The coordinator does not tell Cargo that this output is a Rust library.

Those interfaces remain usable during development. They are not the target architecture or proof of full native compilation. The coordinator does not parse application source or recreate Cargo's dependency graph merely to choose a compiler.

## Existing Cranelift compatibility experiment

Use the [official Cargo profile backend mechanism](https://doc.rust-lang.org/cargo/reference/unstable.html#codegen-backend). Keep the selected Rust toolchain. Do not install a nightly, use `RUSTC_BOOTSTRAP`, change the user's default toolchain, or rewrite project manifests during a build.

Supply the dev/test backend through command-scoped `--config` arguments. Setting `CARGO_PROFILE_*_CODEGEN_BACKEND` in the environment also passes those unstable settings to applications, tests, and their child processes. A nested ordinary Cargo invocation can then fail because it did not request the unstable backend feature. The configured compiler choice belongs to this Cargo command; inherited user settings remain the user's settings. Plans describe that requested command, while runtime qualification and benchmark checks inspect actual per-crate compiler commands.

Cranelift is opt-in. Its [current upstream README](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md) lists incomplete `std::arch` SIMD support and experimental panic unwinding, with narrower platform coverage than rustc. Its standalone distribution also defaults to aborting panics. A near-drop-in compiler is not a universal semantic guarantee.

The first real backend gate found a concrete incompatibility. The rustup preview on nightly-2026-11-07 failed the panic-recovery fixture. Upstream's [`unwinding` feature](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/Cargo.toml) is disabled by default. Without it, the [`catch_unwind` intrinsic](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/src/intrinsics/mod.rs) calls the closure directly, and generated cleanup behavior is incomplete. Keeping Cargo's panic flag alone cannot fix that.

The complementary backend is therefore built explicitly from pinned source with unwind support, using its matching nightly-2026-11-05 compiler. `LUST_CRANELIFT_BACKEND` identifies the resulting absolute library path. The coordinator refuses the stock named default for this path. The setup helper installs the pinned build toolchain only when invoked explicitly; normal compilation installs nothing. Full runtime qualification remains mandatory, and this backend is still not a universal replacement for LLVM.

The coordinator must not change panic strategy, suppress overflow checks, remove features, reduce debug information, or change optimization levels to improve benchmark results. Release and custom profiles retain Cargo's settings. A failed compilation, program, or test returns its failure; the coordinator does not repeat the command under another engine.

`cargo check` usually avoids ordinary native code generation, so changing the backend does not address most of its work. Upstream Lust's own plan records a ts-rust case where Cranelift took 53.8 seconds against 43.0 seconds for rustc's normal backend. That is a useful counterexample to a universal speed claim, not a prediction about other workloads.

## Native coverage and execution qualification

A native coverage claim must identify the exact requested target and dependency closure. It must include:

- Compiler, runtime, adapter, and toolchain identities.
- Target triple, edition, features, profile, panic strategy, overflow checks, and debug settings.
- Manifest and lockfile identities, source dependencies, registry dependencies, proc macros, build scripts, and their observed inputs and outputs.
- Any substitution of a dependency by an adapter and the evidence for its required behavior.
- Acceptance and rejection parity, runtime output and exit status, file effects, test results, and panic/drop behavior.
- Cold, warm, edit, and compile-plus-test timings with the same requested configuration.

A successful Lust compile is insufficient. A fallback on rejection would miss incorrect acceptance, and could silently use reduced features or adapters with different behavior. Qualification must not execute an application's `run` command as a hidden probe. Trusted build-script and proc-macro execution stays explicit through the build process.

Qualification is configuration-specific. A changed feature, adapter, build script, compiler, target, dependency, or semantic assumption invalidates it. A certificate covering yesterday's complete source snapshot does not prove tomorrow's arbitrary edit correct. Warm invalidation rules must be separately tested.

## Existing rustc semantic inspection experiment

An earlier experiment considered a rustc-backed semantic interface for checked, monomorphized bodies, layouts, calling conventions, symbols, drop/unwind operations, intrinsics and linkage. The existing inspection prototype is retained as research and does not implement the native compiler requirement.

That prototype leaves metadata publication and Cargo's contract with rustc. A C ABI experiment cannot establish general Rust ABI compatibility. No executable mapping from its reports to native Lust generation exists. Full native crate and metadata support remain required implementation work.

The first executable step is [`lustr-semantic`](../../crates/lustr-semantic), an
opt-in driver bound to the pinned compiler installation. It obtains actual
monomorphized items, symbol identities, function-call ABI, type layouts, drop
requirements and cleanup edges from rustc queries. The normal backend continues
to produce every artifact. Only owned inspection data leaves the analysis
callback, and reports are published atomically after the entire compiler,
including linking, succeeds. Analysis-only invocations do not force
monomorphization.

The probe refuses mismatched compiler installations and custom compiler
executables that would be bypassed by in-process compilation. Its integration
suite checks ordinary versus inspected artifacts in both directions, compiler
and linker failures, Cargo build scripts and proc macros, configuration
preservation, and no-op behavior. This qualifies inspection on the pinned Linux
host. The report deliberately lacks complete MIR execution semantics and cache
inputs; it cannot authorize native code generation or reuse. Inspection adds
overhead and is kept outside the coordinator's normal route.

Extending Lust's persistent frontend to full Rust must account for macro expansion, coherence, negative lookups, trait candidate changes, generated source, const evaluation and cross-crate metadata. Retained work needs explicit ownership and invalidation rules, plus cold-session equivalence tests.

## Performance decisions

Native expansions must remove enough work to preserve Lust's fast development model after checking, invalidation, code generation, linking and executed tests. Keep compiler dispatch, duplicate analysis and optional compatibility work outside native speed claims. Do not infer a gain from the number of compilation paths.

For a baseline compilation time `T`, if fraction `p` is accelerated by factor `s`, the ideal time before coordination overhead is `T * ((1 - p) + p / s)`. Even infinitely fast code generation cannot remove frontend time. Real benchmarks must include coordination overhead and the runtime of executed tests.

Record cold compilation, fresh-process cache reuse, no-op builds, body edits, signature edits, type/trait changes, macro edits, invalid edits, linking, and compile-plus-test time separately. Measure medians and tails, interleave baseline and candidate, retain raw data, and disclose target hardware and compiler identities. Do not describe an eight-line example as proof for a large workspace.

## Completion criteria

This project is complete only when unchanged full Rust projects compile through the extended native Lust/Lustr implementation, existing Lust-supported behavior retains its native path, artifacts interoperate where promised, and representative equivalent-workload measurements establish the required fast compilation. A rustc fallback, coordinator, a few fixtures or a separately optimized rustc build does not meet that definition.

The implementation sequence and its gates are in [the roadmap](roadmap.md).
