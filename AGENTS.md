# Work on this fork

## Product contract

This fork develops a coordinated compiler set that can compile an unchanged, complete Rust project and reduce its development compilation time. Keep upstream Lust's supported subset, rejection rules, daemon design, and differential testing discipline. Complement it where full Rust requires more machinery. A wrapper that delegates everything to rustc is a compatibility starting point, not completion of this goal.

Use one additional full Rust compiler family before proposing more independent frontends. Add a compiler or split a compilation stage only when equivalent-workload measurements show a useful improvement and conformance tests cover the new interface.

## Correctness rules

- Full Rust support currently comes from the installed Cargo/rustc toolchain. Name that dependency honestly. Do not claim that this fork independently implements all of Rust.
- Never infer semantic compatibility from source keywords, a successful parser run, or a compiler accepting the source. Macros, traits, lifetimes, layouts, generics, drops, panic strategy, and crate metadata interact.
- Lust currently has no Rust rlib format. Do not place its output in Cargo's artifact graph as a Rust library or forge metadata or dep-info. Whole-target Lust compilation and a future verified semantic-IR bridge are separate routes.
- Preserve the project's source, features, target, profiles, Cargo flags, dependency versions, panic strategy, and build-script behavior. Adapter substitutions and reduced features require explicit qualification against the requested Cargo build.
- Never turn compiler errors into success. Never retry an executed program or test automatically through another compiler. Do not change panic behavior to obtain a passing or faster result.
- Keep stdout usable for Cargo JSON and program output. Pass OS arguments without a shell or lossy decoding. Preserve subprocess failures and signals.
- Key any future cache by the compiler/runtime identities, complete semantic inputs, target/profile/features, build-script observations, and transitive dependency identities. An untracked input forces recomputation.

## Work layout and validation

`tools/cargo-lust` is the independently installable coordinator. Its own build must not require tcc, nightly Rust, or registry dependencies. Upstream compiler sources remain in `src/` and `library/`.

Read `docs/compiler-set/design.md` and `docs/compiler-set/roadmap.md` before changing routing or claiming coverage. The upstream `DESIGN.md`, `docs/ARCH.md`, and `docs/PLAN.md` contain historical plans. Check executable code and tests before treating those plans as implemented behavior.

Use real integration tests for the Cargo path, including build scripts, proc macros, full-language constructs, rejection of invalid Rust, and observable runtime behavior. Keep the upstream differential gates for changes to Lust itself. Report which engines and platforms actually ran.

Measure cold builds, no-op builds, body edits, signature edits, rejection latency, and compile-plus-test time separately. Preserve raw samples and exact compiler identities. Report regressions. Do not transfer upstream Lust's speed ratios to this fork, Cranelift, or arbitrary Rust projects.

Keep changes reviewable on a branch. Do not change repository visibility, upstream licensing, or the user's other forks as part of compiler work.
