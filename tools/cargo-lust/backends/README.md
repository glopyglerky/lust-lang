# Cranelift with panic unwinding

The rustup preview backend did not preserve the full-Rust fixture's panic
recovery on nightly-2026-11-07. Upstream compiles its catch-unwind lowering and
cleanup blocks only when the backend itself has the `unwinding` Cargo feature.
Passing `-Cpanic=unwind` to an application cannot add that missing backend code.

The pin in `cranelift.json` selects upstream commit
`8edf4d040acb77048e04b52a379e93dccae888e6` and its declared nightly-2026-11-05.
The setup helper uses the backend's upstream Cargo build recipe with
`--features unwinding`, the locked dependencies, and no optional JIT or
unstable inline-assembly feature. It keeps the toolchain's normal LLVM-built
standard library. It does not use the standalone `cargo-clif` wrapper.

```sh
bash tools/cargo-lust/scripts/build-cranelift.sh
source tools/cargo-lust/target/complement/env.sh
cargo lust --engine cranelift build
```

Setup requires Linux x86_64, Python 3.10 or later, Git, rustup, a C linker, and
network access for the pinned sources, toolchain components and crate packages.
It installs the named nightly and compiler-development components without
changing rustup's global default. `--skip-install` uses an existing installation.
It writes only its owned output directory, apart from the explicit rustup
installation. Its Cargo home is inside that directory.

An existing source checkout must have the exact origin, revision and lockfile,
with no local changes. An output directory belonging to another pin is rejected.
Choose a new `--output-dir` when testing a different pin. Setup relies on Cargo's
own dependency tracking and never treats an old backend binary as a successful
new build.

Setup writes a small bootstrap workspace inside its owned output directory, with
the checked-out backend as its only member and an exact copy of its pinned lock.
This prevents Cargo from treating the backend as part of a surrounding project's
workspace. The upstream source manifest is unchanged.

Before publication, setup compiles and runs a panic-recovery probe that requires
both `catch_unwind` and a destructor during unwinding. Publication uses an
immutable `lib/<sha256>/librustc_codegen_cranelift.so` path, so changed backend
bytes change Cargo's backend argument and cannot reuse an old artifact merely
because the library pathname stayed fixed. Existing different bytes at that
digest path are rejected. `backend.json` records source, lockfile, features,
toolchain, output hash and probe evidence. `env.sh` exports only the pinned
toolchain and exact backend path.

The setup probe is a prerequisite for the repository's larger differential
fixture. It does not qualify all Rust programs, targets, compiler flags or
intrinsics. The backend remains experimental, and performance must be measured
with unwinding enabled.

Inspected upstream sources at the pinned revision:

- [Backend features](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/Cargo.toml).
- [Official build recipe](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/build_system/build_backend.rs).
- [Pinned toolchain](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/rust-toolchain.toml).
- [Catch-unwind lowering](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/src/intrinsics/mod.rs).
- [Cleanup block lowering](https://github.com/rust-lang/rustc_codegen_cranelift/blob/8edf4d040acb77048e04b52a379e93dccae888e6/src/base.rs).
