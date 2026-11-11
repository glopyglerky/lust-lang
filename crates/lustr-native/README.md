# Native Cargo entry point

`lustr-native` is an experimental native compiler. It runs the unchanged Lust
compiler pipeline in the same process. Cargo selects it through `RUSTC`; no
compiler-selection process, rustc fallback, source rewrite, or fake Rust
metadata is involved.

The protected source under `compiler/native` stays unchanged. `src/main.rs`
declares its original modules directly from that tree. Two explicit adapters,
`native_expand.rs` and `native_resolve.rs`, pass Cargo's crate name through the
upstream public `ExpandCtx` and `CrateInput` fields. They preserve the real input
path and source text. This avoids deriving `module_path!()` from `main.rs`.

## Current boundary

The first supported contract is a dependency-free Rust 2024 binary with the
Lust-supported language subset and the development backend. Native `rlib`,
Rust metadata, procedural macro artifacts, test harnesses, other editions,
cross-compilation, arbitrary custom cfg values, release optimization, external
Rust libraries, and unimplemented rustc flags return errors. This compiler is
not a full rustc replacement.

The Cargo probes identify the compiler as `lustr-native 0.1.0`, including its
own build hash and C/TinyCC backend. They do not impersonate rustc or LLVM.
Cargo manifests with a Rust-version requirement greater than this experimental
version can reject it. That limitation must not be hidden by a false version.

`--print=file-names` reports only the native binary artifact. It reports other
crate types as unsupported, using Cargo's documented probe convention. No rlib
or metadata file is created. Unknown output and code-generation requests fail.

The native profile maps panic strategy, overflow checking, debug assertions,
feature cfg values and debug information to upstream Request/Session inputs.
Only optimization level zero is supported. Cargo's incremental directory, when
requested, contains Lust's real unit cache in a `lustr-native` subdirectory.
The application's compile-time environment is captured before that internal
cache selection. It cannot observe the compiler's internal cache path.

For Cargo's `strip=debuginfo` request with debug disabled, the compiler verifies
that the emitted ELF64 little-endian artifact contains no debug sections. It
returns an error if that cannot be established; it does not silently ignore
the request or run a separate stripping process. Other strip modes and stripping
an artifact built with native debug information remain unsupported.

This entry point does not use the upstream whole-request cache or daemon.
Cargo handles unchanged-target freshness. Actual compilations use the native
unit cache. Daemon latency and performance parity with direct `lust` are not
claimed. The separate pristine `lust` executable keeps its original fast path.

## Dependency information

After a successful native compilation, dep-info comes from the actual native
source map, resolver's loaded paths and expansion dependency records. It tracks
included files and `env!`/`option_env!` observations, the compiler executable and
TinyCC runtime files. Native configuration names found in the compiler source
are also recorded, including unset values, so changing an override invalidates
Cargo's freshness before it can reuse an older executable.

Unsupported native configuration overrides are rejected. The accepted cache
controls are `LUST_CACHE_DIR`, `LUST_NO_CACHE`, `LUST_NO_DAEMON`, and
`LUST_CACHE_MAX_BYTES`. Runtime source overrides and front images are not part
of this initial Cargo contract. Paths that Cargo's current dep-info parser
cannot represent safely are rejected rather than emitted ambiguously.
This entry point also requires UTF-8 argument paths and UTF-8 values for valid
environment variable names. It rejects a non-UTF-8 value instead of changing an
`option_env!` observation into `None`.

## Build

Build the matching TinyCC runtime as described by the upstream native package,
then run this separate Cargo workspace:

```sh
LUST_TCC_HOME=/absolute/path/to/native/third_party/tcc \
  cargo build --profile gate --locked -j1 --manifest-path crates/lustr-native/Cargo.toml
```

The result is `crates/lustr-native/target/gate/lustr-native`. This explicit
profile matches upstream's correctness profile, using optimization level 3,
16 code generation units, ThinLTO, no debug information and incremental compiler
builds. Qualification records that build profile; it does not transfer results
from the separate release profile. The local one-unit release build exceeded
available memory, so the preview uses this declared lower-memory profile.

Its runtime
layout matches the existing native package's `lib/lust` tree. The build recipe
imports the original upstream build script, then incorporates both source
trees, build recipes, lockfiles, observed compiler-build settings and TinyCC
files into its own cache identity.
Its registry dependencies match the pinned upstream native lockfile exactly.

The external fixture and qualification runner are under `tests/native-cargo`
and `tools/qualify-native-cargo.py`. They exercise actual Cargo execution,
freshness after source/include/environment edits, runtime behavior, rejection
and independently timed native/oracle builds. Results apply only to the exact
recorded executable and selected profile.

## Protocol references

- [Cargo compiler identity parsing](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/util/rustc.rs)
- [Cargo target and artifact probes](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_context/target_info.rs)
- [Cargo dep-info parsing](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/fingerprint/dep_info.rs)
- [rustc output and invocation arguments](https://doc.rust-lang.org/rustc/command-line-arguments.html)
