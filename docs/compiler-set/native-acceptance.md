# Native compiler acceptance

The required compiler preserves all upstream Lust support at least as fast as
Lust and supplies the rest of Rust at least as fast as stock Cargo/rustc. These
requirements apply together, including mixed programs and concurrent Cargo
builds. There is no permitted performance regression. This document records
the requirement; it does not claim the current compiler meets it.

Both cold and warm compilation must meet their respective floors. Lustr's
implementation may be redesigned as needed; the existing adapter layout is
not a product constraint. Preserve the upstream reference and history so each
redesign can be compared against the actual baseline.

## Design

Cargo resolves dependencies, selects features and targets, runs build scripts,
tracks freshness and schedules compiler jobs. Cargo invokes Lustr directly
through its existing `RUSTC` setting. Lustr implements the compiler protocol,
semantic checks, native artifacts and code generation in that process.

Unchanged Lust remains the implementation for its supported language. Lustr
adds missing behavior through explicit interfaces to that implementation.
Already supported programs must keep their existing fast paths. Additional
features must compose with them rather than selecting a smaller language for
the entire crate. Native library objects are compiled once and reused by their
consumers; copying source into a dependent crate is not separate compilation.

Do not add another scheduler, manifest walker, source classifier, compiler race
or runtime fallback. Avoid preparatory source passes and duplicate analysis.
Extend existing retained state only when its semantic inputs and invalidation
rules are known. Cargo's parallel jobs must not be serialized by a global
compiler lock or multiplied by an independent job scheduler.

## Independent acceptance conditions

| Condition | Required evidence |
| --- | --- |
| Lust compatibility | The actual Lustr entry point accepts unchanged inputs accepted by the pinned upstream compiler, with equivalent diagnostics where required, runtime behavior, artifacts and configuration. Source preservation of a separate `lust` executable is insufficient. |
| Added Rust support | Actual native compilation of valid Rust outside Lust's coverage, including rejection of invalid programs and ownership, panic and drop behavior. A stock compiler is only the comparison oracle. |
| Composition | Existing and added features work in the same crate and in separately compiled dependencies, without feature removal or source rewrites. |
| Concurrency | Independent crates execute concurrently under the same Cargo job budget. A functional process trace must show overlapping compiler processes; a successful parallel build request alone is insufficient. |
| Lust speed floor | The delivered Lustr path is no slower than upstream Lust for each required workload and mode, including retained-state edits. |
| Full Rust speed floor | The same Cargo build using Lustr is no slower than stock Cargo/rustc for each required workload and mode. |

An observed failure is `failed`. Missing, skipped, noisy or insufficient
evidence is `unproved`. Neither state passes. A finite corpus qualifies only
its recorded inputs and environment; it cannot prove every future program,
upstream change or machine. Expanding the corpus must expose gaps rather than
weaken the requirement.

## Measurement rules

Use identical sources, manifests, lockfiles, features, profiles, target triples,
panic behavior, overflow checks and Cargo job counts in each comparison. Bind
the actual executable hashes and build receipts. A different compiler build
profile must be recorded; diagnostic opt1 binaries cannot qualify a delivered
optimized compiler. Use the actual release profile for both native compilers;
Lust's correctness-oriented `gate` profile is not its performance recipe.
Require the observed compiler invocation, rather than trusting a `release`
directory name. Trace functional execution separately from timing.

Record cold compilation, fresh-process cache reuse, no-op requests, body edits,
signature edits, type and macro edits, rejection latency and compilation plus
executed tests separately. No aggregate speedup may hide a slower mode or
workload. Unsupported modes remain missing evidence, not zero-cost work.

Interleave candidate and baseline measurements and retain all raw observations,
including unsuccessful commands, timeouts and regressions. Never retry only
the unfavorable measurements. Use a preregistered conservative one-sided
comparison with a candidate/baseline limit of `1.0`; no tolerance may silently
raise that limit. An interval that cannot establish the speed floor remains
unproved. Report the statistical assumptions and scope with the results.

## Executable gate

`tools/qualify-native-speed.py` exercises the actual native compiler against
upstream Lust and the same Cargo with an external stock Rust oracle. It reuses
the ordinary native Cargo measurements and includes mixed-library concurrency
and the existing unchanged full-Rust workspace. The latter includes build
scripts, procedural macros, async, generics, unsafe operations, panic recovery
and drops. Its current native failure is required evidence of incomplete
coverage, not a reason to replace the fixture.

The native package workflow runs this gate after correctness qualification and
before archive publication. Gate failures still upload diagnostic evidence.
An experimental correctness result or an older preview archive must not be
presented as satisfying the full replacement requirement.

The current compiler lacks full Rust coverage and the Cargo entry point does
not reuse Lust's retained daemon frontend. Those obligations remain open even
if selected cold or no-op samples meet the speed floor. The gate must not
describe those samples as complete native acceptance.
