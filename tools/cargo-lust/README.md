# lustr coordinator

One entry point for upstream Lust and complete Rust projects. The coordinator is written in Rust, has no third-party dependencies, and installs independently of Lust's tcc backend.

The default path preserves the selected Cargo/rustc toolchain. It compiles full Rust because rustc still performs that compilation. This is the compatibility foundation for the compiler set, not an independent full Rust frontend or a speedup claim. Automatic mixed Lust/rustc library compilation is not implemented.

## Install

From this repository:

```sh
cargo install --path tools/cargo-lust --locked
cargo lustr doctor
```

The coordinator requires Rust 1.85 or later. Building the existing `lust` and `lust-core` compilers has the separate prerequisites described in the root README.

Installation provides `lustr`, `cargo-lustr`, and the original `cargo-lust`.
Use `lustr build` or `cargo lustr build` for the same coordinator. Existing
`cargo lust` commands remain compatible. All three entry points share argument
handling and preserve Cargo's output and the executed program's exit status.

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

Build the pinned Cranelift backend with unwind support, then select its matching toolchain. The initial setup helper supports Linux x86_64 and needs Git, rustup, Python 3, and a C build environment:

```sh
bash tools/cargo-lust/scripts/build-cranelift.sh
source tools/cargo-lust/target/complement/env.sh
cargo +nightly-2026-11-05 lust --engine cranelift build
cargo +nightly-2026-11-05 lust --engine cranelift test
```

The helper builds the pinned source with its `unwinding` Cargo feature and records the compiler identity and backend hash. It does not change rustup's default toolchain. The generated `env.sh` selects the pinned toolchain and sets `LUST_CRANELIFT_BACKEND` to the absolute library path. Both exports are explicit changes to the current shell. Ordinary coordinator builds never download or install a toolchain.

Cranelift uses the rustc frontend, so async, unsafe code, procedural macros, generic associated types, and other Rust semantics still go through rustc. The coordinator enables Cargo's official `-Zcodegen-backend` mechanism and selects the library with command-scoped `--config` arguments for the development and test profiles. These settings do not become environment variables inherited by applications, tests, or build scripts that launch Cargo themselves. Existing caller-provided environment settings remain inherited. The coordinator does not select a different panic strategy or alter `RUSTFLAGS`, optimization, features, source, or dependencies.

The coordinator places its configuration immediately after the Cargo subcommand and before the caller's arguments. This keeps later package overrides in the same Cargo configuration list and preserves their normal precedence. A real Cargo regression test checks this ordering independently of the optional backend.

The backend library path must be valid UTF-8 so Cargo can read it as a TOML configuration value. Quotes, backslashes, and control characters are escaped without changing the path. This restriction applies to the backend setting; ordinary command arguments retain their OS representation.

**The stock rustup Cranelift preview is insufficient for this integration.** Our runtime gate found that it failed to catch the fixture's panic. Upstream builds unwind support behind an optional Cargo feature; passing `-Cpanic=unwind` does not add that missing implementation. The coordinator therefore requires the explicit `LUST_CRANELIFT_BACKEND` path instead of selecting the stock backend by name. Keep the matching compiler revision. The backend remains experimental even when a fixture passes.

Upstream also lists incomplete `std::arch` support and panic unwinding with platform restrictions. A backend error remains an error, and a failing program or test is never retried. See the [official backend documentation](https://github.com/rust-lang/rustc_codegen_cranelift/blob/main/Readme.md) and [this fork's verification report](../../docs/compiler-set/verification.md).

The Cranelift choice applies to development `build`, `run`, and `test`. Explicit `--release`, `-r`, grouped release flags such as `-qr`, and `--profile` requests preserve the project's existing Cargo configuration. `check`, `bench`, `doc`, `clippy`, aliases such as `b`, other Cargo commands, and unknown extensions also pass through unchanged. Cargo owns alias expansion because aliases can select a different profile or operation. In particular, this backend does not promise faster `cargo check`.

For a shorter everyday command, add an explicit alias to your project's `.cargo/config.toml`:

```toml
[alias]
fast = ["lust", "--engine", "cranelift"]
```

After sourcing the generated `env.sh`, use `cargo fast build` or `cargo fast test`. Cargo expands this alias before invoking the coordinator. The backend choice remains explicit.

### Keep a package on LLVM

Cargo's package profile overrides can keep a package that needs LLVM on that backend while other packages use the configured Cranelift library. Replace `your-package` with the package's actual name:

```sh
cargo lust --engine cranelift build \
  --config 'profile.dev.package.your-package.codegen-backend="llvm"'
```

This is a manual package choice using the same rustc frontend and toolchain. Generic functions may be instantiated in a dependent crate, so a package override does not guarantee that every use of its code is compiled with LLVM. Qualify the resulting build. The coordinator never retries a failed build automatically.

The optional mixed-backend gate keeps the unchanged fixture library on LLVM and its application and procedural macro on Cranelift. It checks the actual verbose rustc commands and compares the final binary's stdout, stderr, and status with an all-LLVM build. This does not exchange native Lust artifacts or establish arbitrary mixed-backend compatibility.

## Use the existing Lust compiler

Install or build upstream Lust as described in the root README, then use its native target interface:

```sh
cargo lust --engine lust check src/main.rs
cargo lust --engine lust build src/main.rs -o app
cargo lust --engine lust run src/main.rs -- application-arguments
cargo lust --engine lust capabilities --json
```

For a Cargo target, pass Lust's existing `--manifest-path`, `--package`, `--bin`, `--lib`, feature, and target options as appropriate. This route uses Lust's daemon, adapters, runtime, and rejection rules. It does not silently fall back if Lust rejects the source. It cannot build arbitrary Rust libraries for Cargo because Lust has no rlib format.

Upstream Lust targets Rust/Cargo 1.99 and currently assumes that version's build-script artifact layout. Sourcing the complementary backend's environment selects a newer nightly in that shell. When using native Lust's Cargo-target support, select the upstream-compatible installed toolchain explicitly, for example `cargo +1.99.1 lust --engine lust check --manifest-path Cargo.toml`. The coordinator does not apply the benchmark's private Cargo layout compatibility setting to user projects.

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

`plan` and `--explain` produce JSON with the requested engine, selected engine, reason, command, arguments, environment changes and removals, and experimental status. The `codegen_backend` field contains the requested backend library path for the Cranelift route and `null` for other routes. They run no Cargo command, compiler, build script, or application. Invalid UTF-8 arguments are represented as lossless byte objects.

The plan describes the coordinator's requested invocation. Cargo configuration and package overrides can affect the backend used for each crate; qualification checks inspect the actual compiler commands.

`doctor` asks installed Cargo, rustc, and Lust for their versions, lists any stock Cranelift library, and reports the explicitly configured backend path. It installs nothing and builds nothing. It reports availability, not qualification. Cargo configuration and custom wrappers may select a compiler different from the one visible through `PATH` and `RUSTC`; the report states that limitation.

## Verify and measure

```sh
cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline

# Run the actual Lust acceptance/rejection test.
LUST_TEST_LUST="$PWD/target/gate/lust" \
  cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline \
  --test qualification actual_lust -- --ignored --nocapture

# Run actual custom Cranelift parity, mixed-backend, and nested Cargo tests.
source tools/cargo-lust/target/complement/env.sh
cargo +nightly-2026-11-05 test --manifest-path tools/cargo-lust/Cargo.toml \
  --locked --offline --test qualification actual_cranelift -- --ignored --nocapture
```

The ordinary qualification tests use copied, isolated, path-only workspaces. Missing optional compiler engines appear as ignored tests in the default run. They do not count as verified. The optional commands above execute those tests explicitly and fail if their required engine is absent or incorrect.

The nested Cargo gate compiles and runs a separate application through the coordinator. That application launches ordinary Cargo to check another crate, inheriting its environment unchanged. This catches backend settings that leak out of the outer command and make nested Cargo unexpectedly require unstable flags.

The benchmark tool in [`bench/`](bench/) measures equivalent Cargo and coordinated builds. Keep cold, warm, edit, and compile-plus-run results separate. The [architecture](../../docs/compiler-set/design.md) explains why frontend work, generated-program runtime, and coordination costs all matter.

The remaining work is tracked in the [roadmap](../../docs/compiler-set/roadmap.md). Full Rust compilation, automatic fast-path selection, interoperable artifacts, and measured acceleration are separate gates.
