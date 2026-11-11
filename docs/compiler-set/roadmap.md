# Native compiler implementation roadmap

The [native compiler contract](native-contract.md) states the project requirement.
Start with unchanged upstream Lust and add Lustr-owned support for full Rust while
retaining and expanding its fast development compilation. Existing Lust support
must keep using the native path. Lustr supplies the missing native support;
blueFe delivers the combined compiler. Follow the [upstream preservation policy](upstream-lust.md);
every proposed Lust divergence requires both necessity and faster-results evidence.

The earlier Cargo/rustc coordinator work is transitional compatibility tooling.
It must remain accurately identified and cannot close native compiler tasks.
Cargo already selects compilers and toolchains. The native package does not
build or ship an additional coordinator; the rustc command-line and artifact
contract must be implemented in the native compiler.

## Current implementation

| Area | Status |
| --- | --- |
| Native compiler | Exact upstream Lust baseline in `compiler/native`, plus an in-process Lustr-owned Cargo entry point in `crates/lustr-native`. Full native Rust coverage remains incomplete. |
| Native frontend | Existing parser, expansion, resolution, trait/type checking, borrow checking and monomorphization. Remaining language gaps must be reproduced and implemented here. |
| Native libraries and generation | Existing source libraries, adapters and C/TinyCC generation. Missing APIs and semantic support remain native implementation work. |
| Fast edit path | Existing daemon, query reads, retained state and caches. Every expansion needs warm/cold and edit invalidation checks. |
| Cargo/crate support | The experimental native Cargo compiler accepts bounded binary invocations and emits real dependency records. Native Rust rlib/metadata, proc macros, test harnesses and general artifact interoperability remain open. |
| Full Rust compatibility | Supplied by the complete Rust compiler selected directly through Cargo. This is not native Lust coverage. |
| Packaged previews | The current recipe packages pristine Lust, its matching runtime, the experimental in-process native Cargo compiler and a complete Rust toolchain, with no coordinator executable. Full native coverage and rustc command/artifact compatibility remain open. Earlier packages at `595cdc4` included a coordinator; `preview-2026.11.09-2` contained no native Lust. |

## 1. Reproduce native coverage gaps

Use unchanged public project snapshots and native compilation to identify the
first real language, library, macro and crate blockers. Keep exact source,
features, target and profile selections. A static feature scanner is navigation
evidence, not proof that a program compiles or runs correctly.

Acceptance requires reproducible native failures and corresponding rustc oracle
results. Dependency availability and incorrect target selection are separate
failures; they cannot count as language coverage observations.

The [recorded package evidence at `595cdc4`](https://github.com/glopyglerky/lustr/actions/runs/42977250507/artifacts/13836655982)
retains valid Rust rejected natively for unsafe pointers, async/await, generic
associated types, a dynamic iterator, return-position `impl Trait` in a trait,
an external C function, and native rlib emission. Existing generics/traits and
`offset_of!` execute correctly in their bounded fixtures. Invalid type, move
and borrow cases remain rejected. `tests/native-qualification` and
`tools/native-qualification.py` preserve these fixtures, compiler identities and
first failures. They are a starting regression set, not a complete Rust suite.

The broader native regression also reproduces stale whole-request cache results
when whitespace or comments separate `env!`/include macro names from `!` and an
external input changes. This bug is present in pinned upstream Lust. The
[package guide](native-package.md#known-native-cache-limitation) records the
defect and the performance cost of its explicit workaround. Correctness and
whole-compiler performance evidence are still required before admitting a fix.

## 2. Extend the native implementation

Implement missing behavior in Lustr-owned native parsing, resolution, type/borrow
checking, const evaluation, libraries, lowering and generation through explicit
interfaces to unchanged Lust. Any required Lust patch must first pass the separate
necessity and faster-results gate. Do not replace valid
application code with subset-specific rewrites or suppress required checks.

Acceptance requires failing-first regression cases, native artifacts, matching
runtime behavior and invalid-program rejection. Test programs must combine
existing Lust-supported constructs with the new features. rustc is the oracle,
not the compiler producing the candidate's accepted artifact.

## 3. Preserve fast incremental compilation

Specify positive and negative semantic reads for new operations. Track changed
trait candidates, lifetimes, types, constants, macros, generated source and
crate metadata. Retained state must remain valid after edits; uncertain reuse
must recompute native work rather than return stale answers.

Acceptance requires edit sequences comparing warm native answers with cold
native compilation and the Rust oracle. Include accepted-to-rejected and
rejected-to-accepted transitions, correct ownership/drop behavior, and recovery
after errors. Bound retained memory and keep all first failures.

## 4. Complete crates and the Rust language contract

Handle unchanged workspace targets, dependencies, features, build scripts,
procedural macros, tests and documentation. Complete missing Rust language and
standard-library semantics. Implement the required native crate artifacts,
metadata, linkage, layouts, calling conventions and unwinding behavior.

Acceptance requires real projects and Rust compatibility tests, including
cross-crate generics, trait objects, macro-generated code, const evaluation,
unsafe/FFI boundaries, allocation ownership, panic recovery, TLS and atomics.
Native library checking is not rlib emission. A C ABI demonstration or metadata
placeholder does not establish Rust artifact compatibility.

## 5. Deliver the native compiler through blueFe

Package the native compiler, its matching libraries and required TinyCC assets
with correct notices, relocation behavior, source bindings and privacy checks.
Make the native compiler the primary product entry point as its command and
artifact contracts become usable. Preserve existing compatibility tools during
that transition and identify their use explicitly.

Acceptance requires clean installation, offline repeated compilation, actual
native execution, upgrade/cache invalidation, rollback and every advertised
host/target. The existing rustc-based preview is not this delivery.

The Linux x86_64 package gates cover relocation, native execution, runtime asset
discovery, payload identity and offline compatibility. Direct Lust keeps the
original compiler bytes and daemon behavior. The earlier coordinator's measured
dispatch overhead is removed by excluding it from the product and using Cargo's
existing selection. Other hosts, full-project performance and complete native
coverage remain separate requirements.

## 6. Establish full-project speed

Compare native Lust, expanded native Lustr and stock Rust on equivalent work.
Measure cold compilation, warm/no-op requests, body/signature/type/macro edits,
invalid edits, linking and compile-plus-test time separately. Bind exact
compiler identities, configuration, source and cache state.

Acceptance requires retained raw measurements, regressions, representative
unchanged projects and replayable analysis. Preserve Lust's native supported
path and measure the cost of added features. Shared-host intervals and narrow
mathematical arguments must retain their assumptions. An upstream subset ratio
does not prove full Rust speed, and a compiler-build benchmark cannot substitute
for native application compilation.

## Retained compatibility research

Earlier work remains useful evidence within its original scope:

- The [verification report](verification.md) records coordinator and explicit
  native/Cranelift qualification, including failures and runtime checks.
- The [matched CGU16/ThinLTO experiment](measurements/2026-11-09-cgu16-thinlto-v3/README.md)
  measured 108 rustc-based metadata-only root checks. Its 17.9% time reduction
  was against a matched stage2 control. Stock remained faster on all three
  workloads. These are not native Lust/Lustr measurements.
- `lustr-semantic` inspects a pinned rustc's checked information. It emits
  inspection reports, not native executable IR or full cache keys.

The project remains incomplete until the native compiler meets the language,
execution, artifact and performance requirements above. Memory-safety migration
is later work and must preserve the working implementation's behavior.
