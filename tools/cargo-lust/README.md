# cargo-lust

One entry point for upstream Lust and complete Rust projects. The coordinator is written in Rust, has no third-party dependencies, and installs independently of Lust's tcc backend.

The default path preserves the selected Cargo/rustc toolchain. It compiles full Rust because rustc still performs that compilation. This is the compatibility foundation for the compiler set, not an independent full Rust frontend or a speedup claim. Automatic mixed Lust/rustc library compilation is not implemented.

## Install

From this repository:

```sh
cargo install --path tools/cargo-lust --locked
cargo lust doctor
```

The coordinator requires Rust 1.85 or later. Building the existing `lust` and `lust-core` compilers has the separate prerequisites described in the root README.

## Compile a normal Rust project

```sh
cargo lust check --workspace --all-targets
cargo lust build --workspace
cargo lust test --workspace
cargo lust run --bin my-app -- application-arguments
cargo lust build --release
```

These commands retain Cargo's feature resolution, build scripts, proc macros, dependencies, profiles, metadata, artifacts, compiler wrappers, and diagnostics. The default `auto` engine currently selects this compatibility path. It does not inspect Rust source or speculate about whether Lust can compile it.

Coordinator options go before the command. Arguments after the command, including everything after `--`, belong to Cargo or the invoked program. Execution does not pass through a shell. Unix arguments that are not valid UTF-8 remain intact.

## Try the complementary development backend

Install a nightly and its matching Cranelift component explicitly, then choose that toolchain:

```sh
rustup toolchain install nightly-2026-11-07 --profile minimal \
  --component rustc-codegen-cranelift-preview
cargo +nightly-2026-11-07 lust --engine cranelift build
cargo +nightly-2026-11-07 lust --engine cranelift test
```

Cranelift uses the rustc frontend, so async, unsafe code, procedural macros, generic associated types, and other Rust semantics still go through rustc. The coordinator enables Cargo's official `-Zcodegen-backend` mechanism and the development/test profile backend variables. It does not select a different panic strategy or alter `RUSTFLAGS`, optimization, features, source, or dependencies.

This backend is experimental. Upstream lists incomplete `std::arch` support and experimental panic unwinding with platform restrictions. Installing a component does not prove that a project can use it. A backend error remains an error, and a failing program or test is never retried. See the [official backend documentation](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md) and [this fork's verification report](../../docs/compiler-set/verification.md).

The Cranelift choice applies to development `build`, `run`, and `test`. Explicit `--release`, `-r`, grouped release flags such as `-qr`, and `--profile` requests preserve the project's existing Cargo configuration. `check`, `bench`, `doc`, `clippy`, aliases such as `b`, other Cargo commands, and unknown extensions also pass through unchanged. Cargo owns alias expansion because aliases can select a different profile or operation. In particular, this backend does not promise faster `cargo check`.

## Use the existing Lust compiler

Install or build upstream Lust as described in the root README, then use its native target interface:

```sh
cargo lust --engine lust check src/main.rs
cargo lust --engine lust build src/main.rs -o app
cargo lust --engine lust run src/main.rs -- application-arguments
cargo lust --engine lust capabilities --json
```

For a Cargo target, pass Lust's existing `--manifest-path`, `--package`, `--bin`, `--lib`, feature, and target options as appropriate. This route uses Lust's daemon, adapters, runtime, and rejection rules. It does not silently fall back if Lust rejects the source. It cannot build arbitrary Rust libraries for Cargo because Lust has no rlib format.

`LUSTC` selects the Lust executable. Otherwise the coordinator looks beside its own executable and then on `PATH`. `LUST_CARGO` selects the Cargo executable when an embedding tool needs an explicit path; otherwise the inherited `CARGO` executable or `PATH` is used. `RUSTC`, `RUSTDOC`, Cargo wrappers and the selected rustup toolchain remain inherited.

The direct form `cargo-lust +installed-toolchain ...` explicitly invokes `rustup run` for that installed toolchain. It removes a stale inherited `CARGO` path and does not request installation. Combining that form with `LUST_CARGO` is an error because it specifies two conflicting Cargo selections.

## Inspect decisions

```sh
cargo lust plan build --workspace
cargo lust --engine cranelift plan build
cargo lust --engine cranelift plan build --release
cargo lust --engine lust plan check src/main.rs
cargo lust doctor
```

`plan` and `--explain` produce JSON with the requested engine, selected engine, reason, command, arguments, environment changes and removals, and experimental status. They run no Cargo command, compiler, build script, or application. Invalid UTF-8 arguments are represented as lossless byte objects.

`doctor` asks installed Cargo, rustc, and Lust for their versions and checks the selected rustc sysroot for a Cranelift library. It installs nothing and builds nothing. It reports availability, not qualification. Cargo configuration and custom wrappers may select a compiler different from the one visible through `PATH` and `RUSTC`; the report states that limitation.

## Verify and measure

```sh
cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline

# Run the actual Lust acceptance/rejection test.
LUST_TEST_LUST="$PWD/target/gate/lust" \
  cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline \
  --test qualification actual_lust -- --ignored --nocapture

# Run the actual Cranelift full-Rust/panic parity test.
cargo +nightly-2026-11-07 test --manifest-path tools/cargo-lust/Cargo.toml \
  --locked --offline --test qualification actual_cranelift -- --ignored --nocapture
```

The ordinary qualification tests use copied, isolated, path-only workspaces. Missing optional compiler engines appear as ignored tests in the default run. They do not count as verified. The optional commands above execute those tests explicitly and fail if their required engine is absent or incorrect.

The benchmark tool in [`bench/`](bench/) measures equivalent Cargo and coordinated builds. Keep cold, warm, edit, and compile-plus-run results separate. The [architecture](../../docs/compiler-set/design.md) explains why frontend work, generated-program runtime, and coordination costs all matter.

The remaining work is tracked in the [roadmap](../../docs/compiler-set/roadmap.md). Full Rust compilation, automatic fast-path selection, interoperable artifacts, and measured acceleration are separate gates.
