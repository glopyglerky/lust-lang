# lustr semantic inspection

`lustr-semantic` is an executable development tool for inspecting the rustc
semantics that a native Lust backend will need to share. It runs the pinned Rust
compiler in process, reads checked compiler data, continues ordinary metadata
generation, code generation and linking, and writes a report after successful
compilation.

**This tool provides inspection, not a native Lust backend or a compilation
speedup.** The normal rustc backend produces every artifact. No generated C,
Lust object, or alternative runtime participates in these builds. Inspection
adds work and can produce large reports.

## Build and use

The implementation is pinned to `nightly-2026-11-05`, compiler commit
`2d20b42b883dcfc014f3c2c56081eeb6091c2179`. The current integration tests run on
`x86_64-unknown-linux-gnu`.

From the repository root:

```sh
rustup toolchain install nightly-2026-11-05 --profile minimal \
  --component rustc-dev --component rust-src --component llvm-tools
cargo +nightly-2026-11-05 build \
  --manifest-path crates/lustr-semantic/Cargo.toml --locked
```

The package uses the compiler's own `serde_json` through `rustc_private`; it has
no registry dependencies. `rustc-dev` supplies the compiler libraries, and
`llvm-tools` supplies their LLVM link dependency. Its executable embeds runtime
library paths to that compiler installation. Rebuild it if the toolchain moves.

Set an absolute path to the built executable, then run Cargo in the project you
want to inspect:

```sh
probe=/absolute/path/to/lustr/crates/lustr-semantic/target/debug/lustr-semantic
rustc_path=$(rustup which --toolchain nightly-2026-11-05 rustc)
RUSTC="$rustc_path" RUSTC_WORKSPACE_WRAPPER="$probe" \
  LUSTR_SEMANTIC_DIR="$PWD/target/lustr-semantic-reports" \
  cargo +nightly-2026-11-05 build
```

The wrapper inspects workspace compiler invocations, including workspace build
scripts and procedural macros. Cargo retains control of dependency ordering,
features, compilation flags and artifact names. Existing fresh artifacts stay
fresh; the probe emits reports only for invocations that actually run. To
inspect `cargo check`, replace `build` with `check`.

Place reports **inside Cargo's target directory or outside the workspace**.
Files generated elsewhere inside a package can trigger a build script's
default change detection and turn a subsequent no-op build into a rebuild.
Use the actual target directory if the project sets `CARGO_TARGET_DIR`.

For one compiler invocation:

```sh
rustc_path=$(rustup which --toolchain nightly-2026-11-05 rustc)
LUSTR_SEMANTIC_DIR=/tmp/lustr-semantic-reports \
  "$probe" "$rustc_path" --edition=2024 example.rs -o example
```

`LUSTR_SEMANTIC_DIR` is required for inspection, including Cargo's preliminary
compiler queries. Queries such as `--print` and `-vV` retain their compiler
output and produce no semantic report. The probe does not install itself into
the coordinator or alter a project's Cargo configuration.

## Compiler identity and invocation behavior

Before entering `rustc_driver`, the probe asks the supplied compiler for its
commit, host, release and sysroot. They must match the compiler installation
used to build the probe. An explicit `--sysroot` must resolve to the same
canonical directory. A different installation is refused even when it reports
the same Rust commit; a matching version string alone is not an ABI guarantee.
The build script also refuses an unpinned compiler commit.

Inspection requires the supplied executable to resolve canonically to that
installation's actual `bin/rustc`. Symlinks to the actual compiler are accepted.
Custom compiler wrappers and rustup proxies are refused, including through
`PATH`: forwarding the right version queries does not prove that a wrapper
preserves compilation flags. Set `RUSTC` explicitly as shown above. This
restriction prevents in-process compilation from silently bypassing a user's
compiler wrapper.

UTF-8 arguments are passed intact to rustc's entrypoint, which handles response
files and compiler options. The probe writes no inspection text to stdout, so
Cargo's JSON message stream remains parseable. Compiler failures retain their
failure exit status and diagnostics. It does not retry a failed compiler,
linker, test or program.

`rustc_driver` accepts UTF-8 arguments. On Unix, an invocation with a non-UTF-8
argument is delegated with `exec` to the supplied compiler, preserving its
argument bytes, stdout, stderr, signals and exit status. That path produces no
inspection report and makes no in-process compiler ABI claim; the pin and
entrypoint checks apply to inspection, not to this exact passthrough.

Inspection errors or report publication errors fail the explicit probe
invocation. Compiler timing output includes inspection overhead; this tool is
not suitable for measuring the coordinator's compilation speed.

## Inspection format, version 1

Reports use `schema = "lustr.rustc-semantic-inspection"` and
`schema_version = 1`. They contain only owned JSON values; compiler lifetimes,
type pointers and MIR references never leave the analysis callback.

| Section | Data recorded |
| --- | --- |
| Compiler | Pinned commit, host, release, canonical installation |
| Invocation | Original argument vector and working directory |
| Crate | Name, stable crate identifier and crate types |
| Configuration | Target, data layout, panic strategy, optimization level, debug assertions and overflow checking |
| Inputs observed | Source hashes from rustc and dependency crate hashes |
| Codegen units | Compiler-selected units, monomorphized item symbols, linkage, visibility and local-copy status |
| Function ABI | Calling convention, unwind permission, variadic status, argument/return pass modes and attributes |
| Type layouts | Size, alignment, field offsets/order, enum tags and niche encoding, sizedness and backend representation |
| MIR inspection | Instantiated local types, drop requirements, block successors, drop edges and unwind actions |

ABI argument layouts come directly from `FnAbi`; ordinary local layouts come
from rustc's normalized, fully monomorphized layout query. Different generic
instances retain their distinct compiler symbols. The probe records static
layouts and identifies global assembly without attempting to lower it.

The analysis callback runs before metadata encoding. The **local crate hash is
therefore explicitly unavailable**; obtaining it there would violate rustc's
query ordering. Dependency crate hashes are already available. The report does
not claim to capture every semantic input, environment read, file read, or
dependency artifact. `cache_key_complete` is always `false`.

For metadata-only compilation, `configuration.mode` is `analysis-only` and
`codegen_units` is empty. The probe does not force monomorphization in this
mode. A recursive generic program that passes `cargo check` but fails code
generation still behaves that way through the probe.

MIR statements, operand semantics, constant allocations, vtable contents and
complete code-generation obligations are not serialized. Fields ending in
`_debug` are pinned compiler inspection strings. The format is **not executable
IR**, and `executable_ir` is always `false`.

### Check a report's envelope

```sh
"$probe" --validate /path/to/lustr-semantic-123-456.json
```

Validation returns zero silently for a compatible inspection envelope. It
rejects unknown schema versions, compiler/host/installation mismatches,
missing or unknown envelope fields, unsuccessful compilation, claims of
executable IR or a complete cache key, and inconsistent analysis-only mode.
The producer runs this same check before publication.

This validates the envelope and its compatibility with this probe. It does not
authenticate a file, prove that its recorded inputs remain current, or deeply
validate all inspection payload values. Edited JSON cannot authorize native
lowering. A future backend must obtain trusted compiler semantics and complete
the remaining lowering and artifact contracts separately.

### Publication and failures

Each successful inspected invocation writes a unique JSON file. Publication
occurs after the complete rustc entrypoint returns successfully, including
linking. A report is written to a temporary file, flushed, and published with
an atomic, non-overwriting hard link in the same directory. The report
directory must support hard links. Readers see a complete report; concurrent
compiler processes do not overwrite each other's reports.

A compiler or linker failure publishes no new success report, even if
inspection already completed. Previously completed reports remain historical
records of their own invocations. `compilation_succeeded` refers to that
compiler invocation, not the success of the entire Cargo command or the
program's subsequent tests.

## Verification

After building, run:

```sh
python3 crates/lustr-semantic/tests/test_probe.py -v
```

The suite invokes the real pinned compiler. It checks ordinary-rustc to
inspected-artifact interoperability in both directions, including generic
instantiation, large aggregate returns, slices, trait objects, enum niches,
panic catching and destructor execution. It also covers compile and late link
failures, metadata-only compilation, panic/optimization flags, identity and
sysroot refusal, envelope refusal, response files and paths with spaces,
non-UTF-8 delegation, concurrent publication, and a Cargo workspace with a
build script, generated source and a procedural macro. Cargo JSON output and
no-op behavior are checked in that workspace.

The test runner accepts `--probe`, `--rustc` and `--cargo` paths for CI. These
tests establish the inspection seam; they do not establish interoperability
between native Lust-generated artifacts and Rust libraries. Cross-compilation
and distinct host/target runtime compatibility have not been qualified here.

## Next implementation boundary

A native backend can now compare its intended handling against actual pinned
rustc symbols, argument passing, layouts and cleanup structure. The next step
is a narrowly supported lowering path behind the same compiler interface,
with explicit rejection of unsupported MIR operations and a normal rustc
fallback selected before artifact generation. It must preserve metadata,
runtime contracts, drops and unwind behavior, and pass both-direction artifact
tests before the coordinator can route ordinary Rust work to it.

The relevant pinned compiler sources are the
[driver entrypoint and callbacks](https://github.com/rust-lang/rust/blob/2d20b42b883dcfc014f3c2c56081eeb6091c2179/compiler/rustc_driver_impl/src/lib.rs),
[monomorphized items](https://github.com/rust-lang/rust/blob/2d20b42b883dcfc014f3c2c56081eeb6091c2179/compiler/rustc_middle/src/mono.rs),
and [function ABI definitions](https://github.com/rust-lang/rust/blob/2d20b42b883dcfc014f3c2c56081eeb6091c2179/compiler/rustc_target/src/callconv/mod.rs).
