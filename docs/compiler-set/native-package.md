# Native toolchain preview package

This package contains the unchanged upstream Lust compiler, its matching TinyCC
runtime and source libraries, and a pinned complete Rust toolchain for explicit
compatibility. Native compilation calls Lust directly. Cargo uses the selected
Rust compiler directly. The archive contains no Lustr coordinator executables.
Native Rust coverage remains incomplete. Unsupported native input stays rejected;
full Rust compatibility uses the packaged rustc.

## Use

Extract the archive into a directory you own. No installer changes your system
toolchain. The package needs a Linux x86_64 system compatible with Ubuntu 24.04,
a C linker, and C development headers. Only this host is qualified by the
package workflow.

The source-built BlueFe component requires glibc 2.39 or newer and
`libstdc++.so.6` providing `GLIBCXX_3.4.32`, plus `libgcc_s.so.1`. Its Cargo
requires OpenSSL 3, `libssl.so.3` and `libcrypto.so.3`, libcurl,
`libcurl.so.4`, and zlib, `libz.so.1`. These are system libraries, not bundled
copies. Their distribution packages supply the transitive dependencies.
Install the dependencies used by the qualified Ubuntu 24.04 build with:

```sh
sudo apt-get update
sudo apt-get install build-essential libssl-dev libcurl4-openssl-dev zlib1g-dev ca-certificates
```

The manifest records requirements for its actual compatibility origin. The
BlueFe ABI requirements above come from the delivered component's ELF
dependencies and versioned symbols; an older glibc system is not qualified.

Set the package location to its extracted absolute directory:

```sh
LUSTR_ROOT=/absolute/path/to/lustr-native-preview-x86_64-unknown-linux-gnu

# Native compilation of Lust-supported Rust.
"$LUSTR_ROOT/bin/lust" build main.rs -o app
./app
"$LUSTR_ROOT/bin/lust" check main.rs

# Run these commands in an ordinary Cargo project with a lockfile.
RUSTC="$LUSTR_ROOT/bin/rustc" RUSTDOC="$LUSTR_ROOT/bin/rustdoc" \
  "$LUSTR_ROOT/bin/cargo" build --locked
RUSTC="$LUSTR_ROOT/bin/rustc" RUSTDOC="$LUSTR_ROOT/bin/rustdoc" \
  "$LUSTR_ROOT/bin/cargo" test --locked
```

For the combined BlueFe archive, use its extracted
`bluefe-lustr-native-preview-x86_64-unknown-linux-gnu` directory as `LUSTR_ROOT`.
The same commands apply. Verify the archive with its accompanying `SHA256SUMS`
before extraction.

The command-scoped `RUSTC` and `RUSTDOC` select the bundled compiler and
documentation tool without changing the system toolchain. Cargo still applies
the project's configuration, features, profiles and configured wrappers. Do not
set `RUSTC` to `lust`. Native Lust does not yet implement rustc's command and
rlib artifact contracts, so Cargo cannot use it as a general rustc replacement.

Native Lust's client and daemon are the exact upstream executables built from
the recorded source. Their normal cache and daemon behavior is retained,
including the known invalidation defect described below.
The package does not set LUST_LIBRARY or LUST_STD_SRC: upstream treats those
overrides as reasons to avoid its daemon. Runtime assets use upstream's
relative lib/lust discovery instead.

The package builder supports two explicit compatibility origins:

| Archive prefix | Compatibility compiler | Identity record |
| --- | --- | --- |
| `lustr-native-preview` | Official Rust 1.99.1, installed by the standalone Lustr workflow | `lustr-toolchain.json` records `rust_compatibility: "upstream-rust"`, the pinned release, versions and binary hashes. |
| `bluefe-lustr-native-preview` | BlueFe built from its unchanged pinned Rust source and qualified before assembly | `lustr-toolchain.json` records `rust_compatibility: "bluefe"`; `bluefe-package.json` binds the source commit, bootstrap inputs, versions, tool hashes and component qualification. |

Read the delivered manifests for the actual compiler version and source commit.
The combined package's source-built BlueFe compiler is distinct from the Rust
1.99.1 bootstrap used to build native Lust. Assembly checks
BlueFe's recorded versions and tool hashes against the actual component, then
repeats runtime and relocation qualification on the combined package.

## Known native cache limitation

The pinned upstream Lust whole-request cache has a confirmed invalidation bug.
Its [external-input guard](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/cache.rs)
matches literal macro spellings such as `env!` and `include_str!`. Whitespace or
comments before `!`, as in `env !("VALUE")` or
`include_str /* comment */ !("input.txt")`, can evade that guard. A later native
invocation can then reuse a stale result after the environment variable or
included file changes. The affected family is `env!`, `option_env!`, `include!`,
`include_str!` and `include_bytes!`.

The existing regression test
`actual_lust_invalidates_external_reads_with_comments_before_macro_bang`
reproduces this against the pristine baseline. The package records the defect
as unfixed; it does not patch the compiler or change cache defaults.

For an affected invocation, explicitly bypass the daemon and disk cache:

```sh
LUST_NO_DAEMON=1 LUST_NO_CACHE=1 \
  "$LUSTR_ROOT/bin/lust" build main.rs -o app
```

The same environment assignments apply to `lust check`. Keep them scoped to
the affected command. This workaround performs fresh native work and loses
daemon and disk-cache reuse, so it does not preserve the measured warm-build
latency. The ordinary direct commands above retain the upstream fast path.

## Layout and evidence

- bin/lust and bin/lust-core are pristine native Lust.
- bin/cargo, bin/rustc, bin/rustdoc and lib/rustlib are the compatibility toolchain.
- lib/lust contains the pinned TinyCC binary/runtime and Lust source libraries.
- lustr-toolchain.json records component identities and hashes. Its marker
  identifies the package layout and records `runtime_coordinator: false`, system
  requirements and known limitations. It does not prove semantic correctness.
- share/licenses/lustr retains native component and dependency notices.
- share/lustr/source contains the exact Lustr, TinyCC and vendored native
  dependency source archives.

The workflow publishes an installable archive only after source preservation,
native compile/run, explicit unsupported-input rejection,
relocation, and offline compatibility tests pass. The compatibility fixture
covers a workspace, generated source, a procedural macro, async code, generics,
panic catching and exactly-one destructor execution, tests and doctests.

The separate evidence artifact retains all qualification commands, outputs,
binary identities, coverage observations and timing samples. Known native
coverage gaps are recorded as gaps. The native qualification runner's strict
--require-all-native mode requires all of that runner's bounded fixtures to pass
and is not claimed by this preview. Passing those fixtures would still not
establish full Rust coverage.

Native timings compare the pristine build and packaged direct Lust on one
identical supported program. They separately
record process-cold, daemon no-op and body-edit compilation with actual output
checks. Order rotates between lanes. Raw results and regressions remain visible.
This small workload does not prove full-project speed or general preservation
of every upstream ratio; no such claim follows from passing packaging checks.

The [first package run at f710aa1](https://github.com/glopyglerky/lustr/actions/runs/42975730690)
recorded these direct-compiler medians, in milliseconds, for seven observations
per lane and mode:

| Mode | Pristine Lust | Packaged direct Lust |
| --- | ---: | ---: |
| Fresh process | 40.21 | 40.03 |
| Daemon no-op | 4.17 | 4.10 |
| Daemon body edit | 5.73 | 5.56 |

Direct packaged Lust retained the original executable bytes and was close to
the baseline on this fixture. These observations belong to that recorded run,
on its shared CI host, and do not prove exact speed equality or performance on
other programs. They also do not establish correctness of external-input cache
invalidation or timings with the workaround above.
Every subsequent package run retains its own raw measurements and source binding.

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

