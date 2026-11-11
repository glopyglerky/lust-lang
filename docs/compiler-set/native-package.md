# Native Lustr package

The current recipe contains native Lust, the Lustr-owned native Cargo compiler,
Cargo, TinyCC and the required runtime/source assets. Cargo selects Lustr
through `RUSTC`. Stock rustc, rustdoc, a stock Rust sysroot and the historical
coordinator are excluded from the native package.

Full native Rust support is unfinished. The required independent-library gate
currently fails on unsafe Rust support. The workflow retains that failure and
will not upload an installable package while its qualification is unsuccessful.
A compiled executable, source-preservation check or successful archive assembly
alone does not qualify the full replacement.

## Compiler behavior

`bin/lustr-native` contains the unchanged upstream Lust modules and Lustr-owned
interfaces in one process. A dependency-free binary retains its existing native
driver path. Lustr now also emits native library objects and typed metadata and
can resolve a binary's `--extern` dependencies against those artifacts. It does
not compile the producer's source again or invoke another Rust compiler.

The first native library format is an ar archive containing real x86-64 ELF
relocatable objects and `lustr.meta`. Its metadata records public scalar function
signatures, stable crate/function identities, source-location records, profile,
compiler/runtime identities and object corruption checksums. The `.rmeta`
sidecar contains that same semantic metadata, not an empty placeholder. This is
Lustr's format; rustc metadata and Rust ABI interoperability are not implemented.

Current library boundaries are explicit:

- Original public safe free functions at the crate root with unit, bool, char,
  integer and floating-point parameters and results can be exported. Private
  helpers use the native compiler.
- A binary can import separately compiled libraries, including two versions of
  one crate under different Cargo aliases. Function bodies remain solely in the
  library objects. Imported signatures are bodyless semantic declarations.
- Consumer source is parsed unchanged and checked with the upstream type and
  borrow checker. An owned IR step links imported calls through typed native
  function-pointer bindings using the upstream backend's ABI.
- Unsafe Rust, public generic/type/trait/const/macro contracts, transitive library
  imports, consumer module/type/generic items and metadata-only compilation are
  unfinished. Producer statics requiring initialization, generated shared glue
  and unrepresented export attributes also fail explicitly. Public modules,
  re-exports and crate inner attributes are rejected because their caller-facing
  contracts are not yet represented in native metadata.

Unsupported requests remain errors. No stock compiler supplies missing native
coverage. The direct `bin/lust` command retains its original CLI and daemon;
`lustr-native` is the Cargo-facing compiler. Its current Cargo path does not
reuse that daemon, and exact preservation of all Lust speed gains is not proved.

## Selecting the native compiler

For a request within the qualified native scope, Cargo's selection is direct:

```sh
LUSTR_ROOT=/absolute/path/to/lustr-native-preview-x86_64-unknown-linux-gnu
RUSTC="$LUSTR_ROOT/bin/lustr-native" "$LUSTR_ROOT/bin/cargo" build --locked
```

The compiler accepts the Rust 2024 development settings documented in the
[native entry guide](../../crates/lustr-native/README.md). The command does not
rewrite a project or its profile to fit the supported scope. Current failures
on a full Rust project identify implementation work still needed.

BlueFe holds stock Rust source and must select Lustr as its only compiler. Its
[Lustr-only build gate](https://github.com/glopyglerky/blueFe/blob/package/qualified-native-20261111/docs/bluefe/lustr-only-build.md)
uses the unchanged upstream bootstrap's external compiler setting. Building
BlueFe with stock rustc and bundling the result is historical reference work.

## Build and qualification

The package workflow bootstraps the compiler with recorded Rust 1.99.1 inputs,
then keeps that compiler outside the delivery as a test oracle. The native Cargo
compiler uses the recorded optimized `gate` profile. Its receipt binds actual
compiler flags, source, TinyCC inputs and the binary hash. Local opt1 diagnostic
builds are correctness experiments and cannot stand in for this package build.

`tools/package-toolchain.py` defaults to `--compatibility-origin none` and
requires standalone `--cargo-bin` and `--cargo-notices` inputs. It preserves
Cargo's distribution notices, native licenses, source archives and build
receipts. The payload manifest binds files, modes, sizes and relative symlinks.
The native package verifier rejects a stock compiler, coordinator or rustlib
sysroot in that payload.

Qualification relocates the candidate with original runtime locations hidden.
It checks ordinary native Cargo compilation and the independent-library
fixture with a separate Rust oracle. The library fixture tests real artifacts,
producer-source removal, crate-version collisions, type/visibility/unsafe
rules, primitive ABI, imported panic locations, malformed artifacts and Cargo
freshness after edits. Functional process tracing checks compiler execution;
timing samples exclude tracing. Local diagnostics without permitted process
tracing are explicitly marked unqualified.

The original full fixture remains a required, failing test until its unsafe
requirements are implemented. Separate safe-library diagnostic evidence can
establish progress on its stated inputs but cannot make that full gate pass.
The workflow publishes an archive only after all of its required checks pass.
Even passing these bounded checks would not prove complete Rust semantics,
BlueFe bootstrap or universal compilation-speed preservation.

## Platform and remaining upstream defect

The initial format and recipe target Linux x86-64 GNU, with package testing on
Ubuntu 24.04. Native linking requires a C linker and development headers. Cargo
uses its distribution's system-library requirements. Other hosts, release
profiles, procedural macros, the Rust test harness and full Rust coverage are
not qualified by this recipe.

The unchanged `bin/lust` whole-request cache still has the documented defect for
comments or whitespace before external-input macro `!`. The separate
[historical package record](historical-native-compatibility-package.md#known-native-cache-limitation)
retains the reproducer and scoped cache-disable workaround. No source patch
passed the required faster-results admission gate. The owned Cargo path does
not use that whole-request cache.

## Earlier packages

[Historical compatibility packaging](historical-native-compatibility-package.md)
records the old stock-rustc and mixed BlueFe archives and their measurements.
Those artifacts keep their original identities and remain distinct from this
native implementation. Their full Rust acceptance and compiler-build timings
are not native Lustr coverage or native Lustr performance evidence.
