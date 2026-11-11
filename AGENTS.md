# Work on this fork

## Current priority and ownership

The user's later clarification makes native compiler expansion the current
requirement: start with Lust, retain its fast compilation path for everything it
supports, and extend that pipeline through Lustr to cover the full Rust language.
blueFe delivers the combined toolchain. A coordinator that forwards full Rust
to rustc and rustc build tuning do not complete this requirement. Migration
toward memory-safe Rust remains a separate, later optimization goal.

`compiler/native` is the exact upstream Lust baseline, not an unrestricted
Lustr editing area. Keep its complete tracked tree identical to `upstream/lust.pin`.
Lustr-owned expansion, fixtures and helpers belong outside that tree. No generated
overlay, vendored copy rename or build-time patch may conceal a Lust divergence.
Any proposed Lust change needs both a reproducible necessity case and faster
equivalent-workload results before admission. Correctness tests alone are insufficient.
The current root Cargo workspace builds the transitional coordinator in
`crates/lustr`; do not mistake that layout for completed native coverage. The
optional pinned semantic driver is `crates/lustr-semantic`. Keep the primary
README, source map, decisions and divergence records about this fork's actual
behavior. Preserve required upstream compatibility, licensing and source history.
Do not claim complete native Rust or whole-toolchain memory safety prematurely.

Cargo already owns compiler and toolchain selection. The user's latest direction
removes an additional coordinator from the native product path. Do not build or
ship `lustr`, `cargo-lustr` or `cargo-lust` coordinator executables in the native
package. Use direct Lust and Cargo's standard compiler selection. Implement the
rustc command-line and artifact contract in the native compiler before selecting
it as Cargo's full replacement; a launcher does not supply that missing support.

`crates/lustr-native` is a separate experimental native compiler entry point.
It imports the pinned modules directly and compiles in the same process. Its
Lustr-owned input adapters supply Cargo's crate name through existing upstream
interfaces. Keep those adapters scoped to the requested source root. Real
dependency records and explicit rejection of unsupported arguments are part of
the compiler contract. It does not yet supply rlibs, procedural macros, test
harnesses, full Rust coverage, or the upstream daemon's warm path. Its Cargo
workspace is separate from the historical coordinator workspace.

## Product contract

Follow [the native compiler contract](docs/compiler-set/native-contract.md).
Use unchanged upstream Lust for its supported code and implement missing support
in Lustr-owned code through explicit, qualified interfaces. Preserve accepted behavior and differential testing
discipline. Added features must execute through native compilation, with actual
runtime and rejection evidence. Do not rewrite applications into the subset,
disable required checks, or count a rustc fallback as native feature support.

## Correctness rules

- Full Rust support currently comes from the installed Cargo/rustc toolchain. Name that dependency honestly. Do not claim that this fork independently implements all of Rust.
- Never infer semantic compatibility from source keywords, a successful parser run, or a compiler accepting the source. Macros, traits, lifetimes, layouts, generics, drops, panic strategy, and crate metadata interact.
- Lust currently has no Rust rlib format. Do not place its output in Cargo's artifact graph as a Rust library or forge metadata or dep-info. Whole-target Lust compilation and a future verified semantic-IR bridge are separate routes.
- Preserve the project's source, features, target, profiles, Cargo flags, dependency versions, panic strategy, and build-script behavior. Adapter substitutions and reduced features require explicit qualification against the requested Cargo build.
- Never turn compiler errors into success. Never retry an executed program or test automatically through another compiler. Do not change panic behavior to obtain a passing or faster result.
- The stock rustup Cranelift preview failed the real panic-recovery gate. Build the pinned complement with its `unwinding` feature and use the matching compiler plus explicit backend path. A panic flag cannot supply backend code that was not compiled in. Preserve the recovery/drop gate.
- Keep stdout usable for Cargo JSON and program output. Pass OS arguments without a shell or lossy decoding. Preserve subprocess failures and signals.
- Scope compiler backend settings to the current Cargo invocation. Injected `CARGO_PROFILE_*_CODEGEN_BACKEND` environment variables leak into programs and tests and can break nested Cargo commands. Place generated Cargo configuration after the subcommand and before caller arguments so later package overrides preserve it. Retain the real configuration-precedence and nested-Cargo regression gates. Verify actual compiler flags when qualifying a backend; a requested plan or successful program alone is insufficient.
- Key any future cache by the compiler/runtime identities, complete semantic inputs, target/profile/features, build-script observations, and transitive dependency identities. An untracked input forces recomputation.

## Work layout and validation

`crates/lustr` is the independently installable coordinator. Its own build must not require tcc, nightly Rust, or registry dependencies. Required native compiler sources remain in `compiler/native/src/` and
`compiler/native/library/`, as described in the primary README.

Read `docs/compiler-set/design.md` and `docs/compiler-set/roadmap.md` before changing routing or claiming coverage. The upstream `DESIGN.md`, `docs/ARCH.md`, and `docs/PLAN.md` contain historical plans. Check executable code and tests before treating those plans as implemented behavior.

Use real integration tests for the Cargo path, including build scripts, proc macros, full-language constructs, rejection of invalid Rust, and observable runtime behavior. Keep the upstream differential gates for changes to Lust itself. Report which engines and platforms actually ran.

Measure cold builds, no-op builds, body edits, signature edits, rejection latency, and compile-plus-test time separately. Preserve raw samples and exact compiler identities. Report regressions. Do not transfer upstream Lust's speed ratios to this fork, Cranelift, or arbitrary Rust projects.

Keep changes reviewable on a branch. Do not change repository visibility, upstream licensing, or the user's other forks as part of compiler work.
