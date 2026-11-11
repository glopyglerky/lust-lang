# Native Cargo entry point

`lustr-native` is an experimental native compiler. It runs the unchanged Lust
compiler pipeline in the same process. Cargo selects it through `RUSTC`; no
compiler-selection process, rustc fallback, or application source rewrite is
involved. Its first native library format stores real compiled objects and a
typed interface. It does not claim to implement rustc's metadata format.

The protected source under `compiler/native` stays unchanged. `src/main.rs`
declares its original modules directly from that tree. Two explicit adapters,
`native_expand.rs` and `native_resolve.rs`, pass Cargo's crate name through the
upstream public `ExpandCtx` and `CrateInput` fields. They preserve the real input
path and source text. This avoids deriving `module_path!()` from `main.rs`.

## Current boundary

The compiler accepts a bounded Rust 2024 development contract on Linux x86-64.
It is not a full rustc replacement.

| Path | Implemented behavior | Remaining limits |
| --- | --- | --- |
| Binary without `--extern` | Keeps the original Lust pipeline for its accepted programs. The owned frontend composes the additional unsafe function, block and scalar function-pointer contracts with the same native checker and backend. | Raw-pointer operations, unsafe FFI and other unimplemented language features remain errors. |
| Library producer | Checks the source, emits native ELF objects, and writes a real ar `.rlib` with typed `lustr.meta` metadata. `--emit=dep-info,metadata,link` also writes the corresponding native `.rmeta`. | At least one original public scalar free function at the crate root is required. Safe and unsafe call contracts are preserved. Libraries with dependencies, public modules/re-exports, exported generics, types, traits, macros or const functions are not implemented. |
| Binary with `--extern name=path` | Loads native artifacts, resolves their public function signatures, checks the original consumer, and links the recorded objects. Different versions of the same crate have separate native symbols. Local generic helpers retain upstream checking and monomorphization. | Consumer source currently supports root free functions and `use` declarations. Modules, body-local items, crate attributes, and standard-library source splicing are not implemented on this path. A root import such as `use dependency;` or `use dependency as alias;` is also not implemented. Qualified calls and function imports are supported. |

Exported parameters and results can be unit, bool, char, signed or unsigned
integers through 128 bits, pointer-sized integers, `f32`, or `f64`. The native
checker still validates private producer bodies. Consumer calls retain their
checked argument and return types, and the original borrow checker runs before
external calls are lowered to actual imported function pointers. Function
pointer coercion for the supported scalar signatures is included. An unsafe
callable stays unsafe through local bindings, aliases, parameters, returns and
casts. Safe functions can become unsafe pointers; the reverse conversion is
rejected. The additional check runs after native type checking and before
lowering, borrowing checks and code emission.

Local generic helpers can compose with native unsafe functions and imported
scalar functions. Their original type, const and lifetime parameters and
where clauses reach the upstream collector and monomorphizer. The additional
safety check visits generic templates even when no caller instantiates them;
unused invalid helpers do not become accepted through dead-code elimination.
Generic exported metadata and unsafe callable transport through unrepresented
generic types remain unsupported. Templates and trait stubs do not emit
symbols, while real compiled/imported symbol collisions still reject.

The owned syntax parser borrows an unchanged token stream. It allocates a
replacement stream only for recognized extension qualifiers, preserving token
ranges, macro bodies and spans. This removes an unnecessary allocation and
copy on imported safe source; it does not establish whole-compiler speed parity.

An artifact binds its target, crate disambiguator, compiler/runtime identity,
panic strategy, exported signatures, source locations, and object checksums.
The importer rejects incompatible identities and malformed artifacts. It reads
compiled objects rather than reparsing dependency source. A `.rlib` made by
rustc or another Lustr build is not an interchangeable input.

The producer rejects public contracts that its metadata cannot preserve.
Examples include `#[must_use]`, `#[deprecated]`, linkage attributes, and
`#[track_caller]`. The owned syntax layer consumes supported unsafe grammar
tokens from the real expanded token stream and retains their original spans,
AST identities and callable contracts. It does not rewrite input files or
modify the protected parser. Imported unsafe signatures receive the same
call-context checks as local unsafe functions. Unsupported aggregate, alias,
generic and nested function-pointer contracts remain errors. This does not
implement arbitrary unsafe Rust operations or every Rust lint.
Library producers requiring live static initialization or generated shared
glue are also rejected until their cross-crate contracts are implemented.
Producer crate inner attributes are rejected as well. Declaration provenance
is checked against the original AST and resolver identity, so an attributed
module or re-export cannot silently lose its contract during flattening.

Metadata-only compilation, including `cargo check` and early metadata
pipelining, remains unsupported. Native transitive dependencies, procedural
macro artifacts, the Rust test harness, other editions, cross-compilation,
arbitrary custom cfg values, release optimization, and unimplemented rustc
flags also remain explicit failures.

The Cargo probes identify the compiler as `lustr-native 0.1.0`, including its
own build hash and C/TinyCC backend. They do not impersonate rustc or LLVM.
Cargo manifests with a Rust-version requirement greater than this experimental
version can reject it. BlueFe currently reaches that check after its compiler
query succeeds. That limitation must not be hidden by a false version.

The exact query options used by BlueFe's bootstrap, including
`-Zallow-features=`, `-Wrust_2018_idioms` and `-Wunused_lifetimes`, are accepted
for supported introspection requests. They are still rejected for source
compilation until their requested feature/lint behavior exists. Query success
is not a claim that those compilation semantics are implemented.

`--print=file-names` reports binary and native `lib`/`rlib` outputs. It reports
other crate types as unsupported using Cargo's probe convention. The native
metadata is specific to this compiler and is emitted only with an actual
linked library. Unknown output and code-generation requests fail.

Newer Cargo versions also request silent JSON reporting for the forced
`unused_crate_dependencies` lint. This entry point accepts that exact request
and reports unused native dependency names after successful compilation.
Resolved calls, function values, and imports determine that set. A binary
without dependencies reports an empty set. This does not enable arbitrary
forced lints or consumption of rustc library artifacts.

The native profile maps panic strategy, overflow checking, debug assertions,
feature cfg values and debug information to upstream Request/Session inputs.
Only application optimization level zero is supported. On the original binary
path, Cargo's incremental directory contains Lust's real unit cache in a
`lustr-native` subdirectory when requested.
The application's compile-time environment is captured before that internal
cache selection. It cannot observe the compiler's internal cache path.

For Cargo's `strip=debuginfo` request with debug disabled, the compiler verifies
that the emitted ELF64 little-endian artifact contains no debug sections. It
returns an error if that cannot be established; it does not silently ignore
the request or run a separate stripping process. Other strip modes and stripping
an artifact built with native debug information remain unsupported.

Cargo handles unchanged-target freshness. A successful binary with one root
file and no imported artifacts can also reuse the upstream whole-request
cache. Admission checks the actual source map, resolver paths and expansion
dependencies. Environment-reading macros and included files remain outside
this cache even when their spelling contains whitespace or comments. A cache
hit regenerates Cargo dep-info and the requested artifact and unused-extern
reports. It preserves source-dependent checking by invalidating on changed
inputs; rejected or incomplete artifact contracts are never admitted.

The cache binds the complete native invocation, original environment digest,
runtime identity, actual executing compiler inode and timestamps, and caller
umask. It verifies the cached response, executable bytes and permissions before
publishing a result. A malformed record or changed compiler/runtime causes
normal compilation. Cache metadata does not store a copy of the caller's
environment. This cache is currently enabled only on Linux.

This entry point still has no retained frontend across source edits. The
original binary path reuses the native unit cache; library and import paths
emit their native objects again when Cargo executes the compiler. Neither this
request cache nor the separate pristine `lust` executable establishes the full
cold and warm speed requirement. Both speed floors still require qualification
of the actual delivered compiler.

## Dependency information

After a successful native compilation, dep-info comes from the actual native
source map, resolver's loaded paths and expansion dependency records. It tracks
included files and `env!`/`option_env!` observations, the compiler executable and
TinyCC runtime files. Imported `.rlib` paths are also dependencies, so a changed
artifact invalidates its consumer. Native configuration names found in the compiler source
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

Build the matching TinyCC runtime as described by the upstream native package.
Commit the source and build helper, then build with a receipt. Use new target
and receipt paths outside the checkout:

```sh
python3 tools/build-native-cargo.py \
  --cargo "$(rustup which cargo)" \
  --rustc "$(rustup which rustc)" \
  --tcc-root "$PWD/compiler/native/third_party/tcc" \
  --target-dir /absolute/new/native-cargo-target \
  --receipt /absolute/new/native-cargo-build.json
```

The result is `native-cargo-target/x86_64-unknown-linux-gnu/release/lustr-native`.
The helper selects the actual bootstrap executable and verifies Cargo's verbose
compiler invocation. It requires optimization level 3,
one code generation unit, ThinLTO, line-table debug information and no incremental
compiler build, matching upstream's release profile. Upstream reserves its
`gate` profile for correctness work and explicitly excludes it from timing.
The version-2 receipt rejects a gate-profile executable. It binds the resulting
executable hash and version to the source commit, unchanged native tree, helper,
bootstrap version, TinyCC inputs and observed compiler command. Its adjacent
`.cargo.log` retains the build output with command environment assignments
removed. Packaging requires both files:

```sh
python3 tools/package-toolchain.py \
  --output dist/lustr-native-preview-x86_64-unknown-linux-gnu \
  --native-bin compiler/native/target/release \
  --native-cargo-bin /absolute/new/native-cargo-target/x86_64-unknown-linux-gnu/release/lustr-native \
  --native-cargo-receipt /absolute/new/native-cargo-build.json \
  --compatibility-origin none \
  --cargo-bin /absolute/cargo-distribution/bin/cargo \
  --cargo-notices /absolute/cargo-distribution \
  --vendor-dir /absolute/path/to/native-vendor
```

The packager verifies the receipt against the supplied executable, current
source and TinyCC files. It copies the receipt and log into the package.
The standalone Cargo distribution must provide `LICENSE-MIT`, `LICENSE-APACHE`,
and `LICENSE-THIRD-PARTY` in the notice directory. This native-only payload
contains Cargo, `lustr-native`, pristine Lust, and their native runtime and
sources. It excludes stock rustc, a rustlib sysroot, and compiler coordinators.
The compiler bootstrap and separate rustc test oracle remain outside the
payload. An independently built binary cannot acquire a release-profile claim
from its filename or the packaging command. Qualification applies to the
recorded executable and settings.

Historical local attempts with the release and gate profiles received SIGKILL.
The cause was not established. A separate optimization-level-1 compiler passed
correctness probes. The native library implementation subsequently built with
the gate profile and passed its correctness checks in CI at `c3d57f6`, but its
strict speed and full-Rust acceptance failed. That record remains diagnostic
for the prior profile; new qualification must build the actual release recipe.

Its runtime
layout matches the existing native package's `lib/lust` tree. The build recipe
imports the original upstream build script, then incorporates both source
trees, build recipes, lockfiles, observed compiler-build settings and TinyCC
files into its own cache identity.
Its registry dependencies match the pinned upstream native lockfile exactly.

## Qualification

`tests/native-cargo` and `tools/qualify-native-cargo.py` exercise ordinary Cargo
execution, freshness after source/include/environment edits, runtime behavior,
rejection, and separately timed native/oracle builds.

`tests/native-library` and `tools/qualify-native-library.py` exercise independent
library compilation through Cargo, source-hidden consumers, colliding crate
names, scalar ABI behavior, visibility and type errors, imported overflow
locations, malformed artifacts, and rebuilds after edits. Its full fixture
also requires unsafe functions and calls. The expanded gate checks lexical
unsafe scope, safe/unsafe pointer conversions, shadowing, lint order and
closure return contracts without removing any original requirement.
`tools/check-native-unsafe-regressions.py` retains malformed-grammar and
safety-analysis regressions found during review. `tools/test-native-protocol.py`
checks the actual BlueFe query and rejects unsupported compilation requests.
A separate safe-library diagnostic keeps its original scope.

`tools/check-native-mixed-generics.py` compares mixed generic/unsafe programs
with stock Rust, checks invalid unused templates, compiles private generic
library helpers, hides producer source before consumers compile, and retains
real imported-symbol collision rejection. `tools/qualify-native-speed.py`
enforces the [acceptance contract](../../docs/compiler-set/native-acceptance.md)
against the actual native entry point. Missing coverage and measurements
remain unproved and block publication.

Local builds made with optimization level 1 and LTO disabled are diagnostic
only. They do not satisfy the recorded package profile. Process tracing is
also required for final library qualification; a run that cannot trace is
marked unqualified. No package is a complete Rust replacement merely because
these bounded fixtures pass.

## Protocol references

- [Cargo compiler identity parsing](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/util/rustc.rs)
- [Cargo target and artifact probes](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/build_context/target_info.rs)
- [Cargo dep-info parsing](https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/src/compiler/fingerprint/dep_info.rs)
- [rustc output and invocation arguments](https://doc.rust-lang.org/rustc/command-line-arguments.html)
