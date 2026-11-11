# Native toolchain preview package

This package contains the unchanged upstream Lust compiler, its matching TinyCC
runtime and source libraries, Lustr's coordinator, and a pinned complete Rust
toolchain for explicit compatibility. It is a usable compiler set for Linux
x86_64. It is not a completed native replacement for rustc: unsupported native
Rust stays rejected and full Rust compatibility still uses the packaged rustc.

## Use

Extract the archive into a directory you own. No installer changes your system
toolchain. The package needs a Linux x86_64 system compatible with Ubuntu 24.04,
a C linker, and C development headers (for example, libc6-dev and build-essential
on Ubuntu). Only this host is qualified by the package workflow.

Set the package location to its extracted absolute directory:

```sh
LUSTR_ROOT=/absolute/path/to/lustr-native-preview-x86_64-unknown-linux-gnu
export PATH="$LUSTR_ROOT/bin:$PATH"

# Native Lust subset: no compiler fallback.
lustr --engine lust build main.rs -o app
./app
lustr --engine lust check main.rs

# Full Rust compatibility using the packaged Rust compiler.
lustr --bluefe-root "$LUSTR_ROOT" --engine rustc build --locked
lustr --bluefe-root "$LUSTR_ROOT" --engine rustc test --locked
```

The packaged coordinator can discover its colocated toolchain when no explicit
compiler selection overrides it. The explicit root above makes selection
reviewable. Existing explicit toolchain, compiler and wrapper selections remain
meaningful; the coordinator's help describes their precedence.

Native Lust's client and daemon are the exact upstream executables built from
the recorded source. Their normal cache and daemon behavior is retained.
The package does not set LUST_LIBRARY or LUST_STD_SRC: upstream treats those
overrides as reasons to avoid its daemon. Runtime assets use upstream's
relative lib/lust discovery instead.

The compatibility toolchain in this recipe is official Rust 1.99.1.
It is not described as a fresh BlueFe source build. The package manifest records
its complete version identity and binary hashes. A future BlueFe-built package
must identify and qualify its own compiler source and bootstrap inputs.

## Layout and evidence

- bin/lust and bin/lust-core are pristine native Lust.
- bin/lustr, bin/cargo-lustr and bin/cargo-lust are the coordinator.
- bin/cargo, bin/rustc, bin/rustdoc and lib/rustlib are the compatibility toolchain.
- lib/lust contains the pinned TinyCC binary/runtime and Lust source libraries.
- lustr-toolchain.json records component identities and hashes. Its marker
  identifies the package layout; it does not prove semantic correctness.
- share/licenses/lustr retains native component and dependency notices.
- share/lustr/source contains the exact Lustr, TinyCC and vendored native
  dependency source archives.

The workflow publishes an installable archive only after source preservation,
coordinator tests, native compile/run, explicit unsupported-input rejection,
relocation, and offline compatibility tests pass. The compatibility fixture
covers a workspace, generated source, a procedural macro, async code, generics,
panic catching and exactly-one destructor execution, tests and doctests.

The separate evidence artifact retains all qualification commands, outputs,
binary identities, coverage observations and timing samples. Known native
coverage gaps are recorded as gaps. The native qualification runner's strict
--require-all-native mode requires all of that runner's bounded fixtures to pass
and is not claimed by this preview. Passing those fixtures would still not
establish full Rust coverage.

Native timings compare the pristine build, packaged direct Lust, and the
packaged Lustr native route on one identical supported program. They separately
record process-cold, daemon no-op and body-edit compilation with actual output
checks. Order rotates between lanes. Raw results and regressions remain visible.
This small workload does not prove full-project speed or general preservation
of every upstream ratio; no such claim follows from passing packaging checks.

The archive checksum binds the delivered bytes; the manifest binds the recorded
components. Neither substitutes for a trusted distribution channel. Replacing a
package means extracting a new directory and selecting it explicitly. The
previous directory remains available for rollback. Compiler identities retain
upstream's cache and daemon separation.

## Rebuild the native compiler

The source bundle includes the unchanged compiler and dependency sources.
Extract source archives into separate directories, configure TinyCC with an
absolute installation prefix, and build it with the system C compiler:

```sh
mkdir lustr-src tinycc-src vendor
tar -xzf "$LUSTR_ROOT/share/lustr/source/lustr.tar.gz" -C lustr-src
tar -xzf "$LUSTR_ROOT/share/lustr/source/tinycc.tar.gz" -C tinycc-src
tar -xzf "$LUSTR_ROOT/share/lustr/source/vendor.tar.gz" -C vendor
cd tinycc-src
./configure --prefix=/absolute/path/to/rebuilt-tcc
make
make install
```

In the extracted Lustr root, configure Cargo's crates.io replacement to the
absolute vendor directory, using the standard Cargo vendored-sources format.
Then build compiler/native/Cargo.toml with --release --locked --offline and
LUST_TCC_HOME pointing at the rebuilt TinyCC installation. No upstream source
patch is applied. To replace TinyCC in an installed package, rebuild lust-core
against that library as well as replacing the matching runtime assets.

The package does not supply system libc headers or link libraries. Native
release-mode operations may invoke external tools according to upstream Lust's
own contract; the default native development path is the qualified package path.

