# Fork ownership and divergences

Lustr and BlueFe are projects with their own entry points, source maps and design
decisions. Required upstream implementation remains where it is needed for a
working compiler and for reliable updates. Inherited branding and historical
research are identified rather than pasted into the primary project README.

## Lustr

The root Cargo workspace builds `crates/lustr`. It has no registry or C build
dependencies and its Rust source forbids unsafe code. The native component lives
in `compiler/native` with a separate workspace, fixtures, runtime assets and build
helpers. This preserves the native compiler's relative-path contracts while
removing those dependencies from the coordinator's default build.

`crates/lustr-semantic` remains independent because its rustc-private interface
requires a different pinned development toolchain. It is not part of the normal
BlueFe application build. The stock GitHub workflows remain at `.github` because
GitHub discovers workflows there; native steps select their component directory.

The native package is named `lustr-native`. Its existing `lust`/`lust-core`
commands and protocol names remain compatibility interfaces. The optional pinned
ts-rust source is a native qualification fixture, not another product entry point.
Upstream licenses and source history remain intact.

Current fork changes include explicit BlueFe selection; corrected Unix startup
and variadic FFI; command-scoped backend selection; panic and bidirectional Rust
ABI gates; Cargo build-script layout handling; native cache invalidation repairs;
and performance evidence tied to exact tools and source. The verification record
separates historical experiments from current qualifications.

## BlueFe

BlueFe retains the Rust compiler and bootstrap layout because the current
compiler, standard libraries, platform support, release tools and upstream sync
use those paths. That compiler source is the working BlueFe implementation.
Removing it before a replacement exists would remove full Rust support.

BlueFe's owned integration is visible through its primary README, root Lustr
submodule and bootstrap changes. It pins Lustr, uses the coordinator for ordinary
Cargo build operations, updates stage0 through its native generator, and enables
unwinding when explicitly building its in-tree Cranelift backend. Default LLVM
compilation and specialized upstream tool launchers retain their tested roles.

## Iteration policy

Make the working path usable before replacing compiler internals. Every inherited
component needs an identified role, preserved license, version boundary and a
reason to remain. Remove or replace it when equivalent functionality and focused
qualification exist. Keep source movement separate from semantic changes in
review and report any changed build or installation contracts.

Full native Rust support, persistent full-Rust analysis, native rlib publishing,
and migration toward memory-safe Rust remain future goals. TinyCC and LLVM are
current external code-generation dependencies. Safe Rust at a command boundary
does not prove their implementation safe. Track migrations and validate behavior
as they land; do not use the future goal to block a working toolchain.

The same ownership and evidence policy applies when working on the user's other
forks. Public repositories can supply frozen performance workloads without
changing their source, default branches, services or deployment state.

## Retired native experiments

The old `compiler/native/spikes` directory contained standalone C/FFI probes
and their drivers. No active compiler, manifest, CI gate or qualification helper
references that directory. These unneeded inherited experiments were removed
from the default source tree. Their complete source and results remain in
[the prior source snapshot](https://github.com/glopyglerky/lustr/tree/39349b63b976e76702ca14fbb5f19718eaf0ed2d/compiler/native/spikes).

Required non-Rust code remains explicit: the current native compiler's C runtime
and generation path, the TinyCC interface, C ABI qualification fixtures, and
build/test/research helpers used by current development. Rust source, licenses
and the relevant compiler fixtures are retained. These exceptions preserve the
working toolchain while the native Rust migration remains future work.
