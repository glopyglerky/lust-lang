# Native compiler requirement

The user clarified the project requirement after the rustc-based preview work.
This contract supersedes the earlier plan to treat a Cargo coordinator plus a
full Rust compatibility compiler as the project's implementation.

## Required behavior

Lust is the starting compiler. Preserve its upstream provenance and the working
Lustr fixes maintained in this fork. Everything it already supports must retain
its native path. Lustr-owned code supplies the remaining Rust language, library
and crate behavior through qualified interfaces. blueFe delivers the combined
compiler.

The [source preservation policy](upstream-lust.md) records upstream and fork
identities separately. Review native changes with regression evidence and update
the accepted fork pin together with their record. Correctness repairs may change
upstream-derived files without proving an unrelated speedup. Keep performance
claims tied to measured equivalent work, and do not hide generated patches.

The intended result accepts unchanged full Rust projects while retaining and
extending Lust's fast development compilation. Full language coverage and
equivalent-workload speed are both requirements. Neither follows from a
wrapper, a successful parse, an optimized rustc build, or a few fixtures.

The integration must compose existing Lust-supported code and Lustr's additions
in the same program. Its native extension interface is not implemented yet.
Do not replace that work with source keyword guesses, a rustc wrapper, or an
unproved edit to the upstream compiler.

## Transitional components

The current `crates/lustr` coordinator's `auto` mode selects Cargo/rustc.
The published `preview-2026.11.09-2` packages contain no native Lust compiler.
Their full Rust support and performance evidence belong to the rustc
compatibility path. They do not satisfy this native compiler requirement.

Keep existing compatibility tools usable while the replacement is developed.
Their fallback must be explicit and must not count as native language coverage,
native acceleration, or project completion. rustc remains useful as a bootstrap
compiler and a differential oracle. Its optional Cranelift, frontend scheduling
and compiler-build experiments are separate research.

The native compiler currently emits C/TinyCC development artifacts. It has no
Rust rlib emitter, and its language and standard-library support remain
incomplete. These are implementation gaps, not permission to reduce the full
Rust requirement. A C object and an empty metadata file cannot substitute for
Rust library metadata and ABI compatibility.

## Evidence required for each expansion

- First reproduce valid Rust rejected by native Lust, or a native behavior
  differing from rustc. Preserve the source and first failure.
- Implement the expansion in Lustr-owned code. Do not change the application to
  avoid the feature or route its acceptance through rustc. Record necessary
  changes to upstream-derived code with their tests and source provenance.
- Check native compilation, emitted artifacts and runtime results against the
  oracle. Retain invalid-program rejection, ownership, panic and drop checks.
- Test mixed programs containing existing and new native features. Check warm
  daemon answers against cold native compilation after relevant edits.
- Measure native cold, warm, no-op, body, signature, type and invalid edits
  separately. Bind source, compiler, configuration and cache state. Retain
  failures and regressions. Compiler-build and rustc-wrapper timings cannot
  stand in for these measurements.

No universal speed ratio or proof of all Rust semantics is claimed by this
contract. The requirement remains open until the actual compiler and its
representative full-project evidence meet it. Memory-safety conversion remains
future work and must preserve the working compiler's behavior.
