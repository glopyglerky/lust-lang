# Fork ownership and divergences

## Source baseline

Lustr's inherited compiler is the complete, unchanged Lust tree pinned in
[`upstream/lust.pin`](../../upstream/lust.pin). The source restoration at
[`82d77539`](https://github.com/glopyglerky/lustr/commit/82d77539680fec9cf822ca8231d1abb985069c48)
made `compiler/native` identical to upstream revision
`240e08d91d4aed0299639f6f11fd522912b248f9`. Its Cargo package is named `lust`,
and its executables are `lust` and `lust-core`.

The root Cargo workspace excludes `compiler/native`; the native compiler keeps
its upstream manifest, build recipe, dependencies, runtime, libraries, fixtures,
documentation, research probes, and licenses. In particular,
`compiler/native/spikes` is present as part of that unchanged upstream tree.

Lustr-owned expansion and qualification belong outside the protected tree.
Every proposed Lust divergence requires both a reproducible necessity case and
faster equivalent-workload evidence under the
[upstream preservation policy](upstream-lust.md). No exception is admitted by
this document. Source relocation, a correctness fix, or a passing benchmark of
a different compiler configuration does not supply that evidence.

## Lustr-owned implementation

`crates/lustr` is the transitional coordinator. Its build has no registry or C
build dependencies, and its Rust source forbids unsafe code. The automatic
route uses Cargo/rustc; explicit Lust selection invokes a whole native target.
Command-scoped backend settings, explicit BlueFe selection, and the compatibility
qualification helpers are retained outside upstream Lust.

`crates/lustr-semantic` remains an independent, pinned development tool. It
inspects rustc's checked information while rustc produces every artifact. Its
reports are neither executable native IR nor complete cache keys. It does not
implement the missing native extension interface.

The native package qualification fixtures under
[`tests/native-qualification`](../../tests/native-qualification) run native Lust
and the Rust oracle independently. Their report preserves native coverage gaps
and does not count the compatibility compiler's acceptance as native support.

Full native Rust support, native rlib emission, and the extension interface remain
unfinished. The [native contract](native-contract.md) and
[roadmap](roadmap.md) define those obligations. The coordinator and bounded
qualification fixtures do not close them.

## Historical Lust changes

The source snapshot before restoration,
[`fb4c4bc7`](https://github.com/glopyglerky/lustr/tree/fb4c4bc79a1d17f1c0f7cdcb47dced6d0f88c316),
contains the earlier `lustr-native` package identity, startup and variadic FFI
changes, Cargo build-script handling, cache invalidation repairs, and associated
tests. Those differences are preserved in Git history and are not current
upstream compiler changes. Their correctness evidence remains historical;
restoring them would require the separate necessity and faster-results admission.

The [verification record](verification.md) identifies historical native and
compatibility experiments. Statements about a recorded run apply to that run's
source and tool identities, not automatically to the current source baseline.

## BlueFe

BlueFe preserves its pinned upstream Rust compiler, libraries, bootstrap, and
stage0 inputs. At
[`f0f00135`](https://github.com/glopyglerky/blueFe/commit/f0f00135a69fdf718ad73aaaecbddeacf36f2b74),
the full `compiler/`, `library/`, `src/`, and `tests/` trees and bootstrap entry
points match upstream `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`.

The earlier bootstrap coordinator injection, custom stage0 selection, compiler
compatibility patches, and explicit Cranelift unwinding configuration are no
longer active source changes. BlueFe's Lustr gitlink, project documentation, and
source-preservation tooling remain separate integration material. Historical
preview binaries retain their original provenance; source equality does not
qualify a new compiler build or establish performance.

## Retained performance research

The [CGU16/ThinLTO study](measurements/2026-11-09-cgu16-thinlto-v3/README.md)
compares rustc-based stage2 packages built from the same source and Cargo. It
changes compiler CGU256/off-LTO to CGU16/ThinLTO, retaining std CGU256 and project
settings. The numerical decision passed for three dependency-primed,
metadata-only roots; stock Rust remained faster on those cases.

That result does not isolate historical source patches, measure full application
builds, establish native Lust/Lustr coverage, or prove preservation of Lust's
fast path. The [study arguments](proofs/cgu16-thinlto.md) retain their mathematical
and statistical assumptions. TinyCC and LLVM remain external code-generation
dependencies; whole-toolchain memory safety remains separate future work.
