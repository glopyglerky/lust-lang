# Native compiler CI and performance evidence

Lustr must meet both cold and warm speed floors. Upstream Lust is the reference
for its accepted Rust subset. Stock Cargo/rustc is the reference for the Rust
Lustr adds. Each required workload and mode must pass independently, with
equivalent inputs and observable behavior. Faster cases do not compensate for
slower ones. The [acceptance contract](native-acceptance.md) remains binding.

## Automatic workflow

The [native workflow](../../.github/workflows/native-package.yml) has one job on
GitHub's standard Ubuntu runner. It builds pristine Lust and native Lustr once,
then uses those same executables for correctness, package relocation and the
sequential comparison campaign. Keeping this work together avoids a second
compiler build or a new artifact-transfer and receipt-rebinding protocol.

Every pull request triggers this workflow. Native package branch pushes and
pushes to `next` also trigger it. The primary gate has no path filters because
[GitHub limits the diff used for path filtering](https://docs.github.com/en/actions/how-tos/troubleshoot-workflows#filtering-and-diff-limits).
A broad compiler change must not silently miss acceptance. This costs a full
run for documentation-only changes but avoids a second change-classification
system. Branch/PR concurrency removes duplicate active runs. A weekly full run
provides a backstop, and `workflow_dispatch` permits a full run on demand. GitHub
schedules activate only when this workflow is on the repository's default
branch; a draft branch does not activate its schedule.

Each primary run also checks the upstream repository's current default-branch
HEAD against `upstream/lust.pin`, retaining the advertised ref, commit and raw
Git response. A moved, unavailable or ambiguous HEAD blocks the build and
archive. This read-only check does not update the pin or donor source. The pin
currently matches the default branch's `240e08d9`; future movement requires a
source update and fresh qualification. A
[live check on 2026-11-11](measurements/2026-11-11-native-cache/upstream-head/report.json)
retains the exact matching response. The weekly and manual triggers cover
periods without local changes once the workflow reaches the default branch.

The existing upstream differential, daemon, sanitizer, fuzz and benchmark-smoke
checks remain on Linux and macOS in the [upstream workflow](../../.github/workflows/ci.yml).
That workflow runs when protected upstream source, its pin, its guard or the
workflow changes. It also has weekly and manual full runs. Lustr-owned changes
continue to run the native compiler gates instead of rebuilding the unchanged
standalone donor on both operating systems for every documentation update.

### Compiler builds and caches

Performance runs require actual release compiler builds. The native build
receipt verifies the observed bootstrap invocation and executable hash, not a
directory name or a requested profile. The native release settings match the
donor's release optimization policy. The old gate profile used more codegen
units and incremental compilation, so its historical measurements cannot
qualify release performance. The original reports remain unchanged and the
summary labels them as historical gate-profile builds.

The native workflow caches only Cargo registry and Git dependency downloads,
keyed by runner OS, architecture, pinned bootstrap version and both compiler
lockfiles. Cargo still checks the locked dependencies. It does not restore
compiled targets, performance samples, runtime caches, executables or receipts.
Source, compiler and runtime identities are rechecked on each run. Cache writes
happen after a successful build on non-PR events, before any later acceptance
failure, so an incomplete compiler does not prevent useful dependency caching.

Native build targets, reports and package output live outside the checkout.
Source-provenance verification continues to require a clean source tree.

### Correctness and measurement

Once the build succeeds, independent correctness checks and the speed campaign
continue even if an earlier independent check fails. They retain their failure
status. A failed producer does not authorize its dependent relocation check.
The workflow uses no `continue-on-error` to authorize package publication.

The campaign compares both engines sequentially on the same runner. It
alternates the order of each pair and retains every observation, including
errors, timeouts and slower pairs. Cold samples get separate initially empty
artifact, unit and runtime temporary directories. Warm samples retain their
own state. Direct cold measurements include both caching disabled and caching
enabled with empty caches. The latter includes first admission and store cost;
an unchanged-cache result cannot substitute for it. OS page caches are not
flushed; the report records that limitation.
Functional process tracing is separate from timing. It verifies actual engine
execution and overlapping Cargo compiler jobs without adding tracing cost to
the measured samples.

The request-cache correctness check uses a private instrumented TinyCC helper
and deliberately disables it after a successful build. This proves an admitted
unchanged request reuses its result while still producing Cargo dependency
information and artifact events. Corruption, changed compiler identity,
permissions and external inputs must force normal compilation or a real error.
An enclosing manifest and build script must never make cache storage or lookup
invoke Cargo again; the check records any attempted invocation with a sentinel.
Those fault-injection runs never supply production timing samples.

After the timing campaign, a separate functional audit compares the native
Cargo entry point with Lust on all 2,293 top-level accepted fixtures in the
pinned `pass`, `unwind` and `release` directories. Both compilers attempt every
selected source unchanged. The audit checks native dep-info and compares
program output, exit status and signals using only upstream's existing masks.
It uses the assembled package, snapshots its executable and runtime support
files, and binds their hashes to the same release binaries used for timing.
Two isolated worker processes keep this compatibility check short; their
durations are never performance evidence. It runs even when the speed gate or
relocation check fails, provided package assembly succeeded.

The explicit fixture matrix uses unwind semantics with overflow and debug
checks enabled for `pass` and `unwind`, and disabled checks for `release`.
Release semantics for the other suites, abort profiles, failure/rejection
diagnostics, alternative backends, cross-target configurations and the other
upstream gates remain uncovered by this audit. Passing it does not prove the
complete upstream accepted-input requirement. Missing, duplicate, unattempted,
unstable or failed cases block its success. Raw logs, per-case results, input
and artifact hashes, storage observations and collection errors are retained;
per-case executables are removed after recording their hashes.

GitHub-hosted runners are not noise-free. Reports retain host identity,
available CPU affinity, raw durations, paired ratios and uncertainty. A missing
or inconclusive result does not pass. Neither statistical confidence nor this
finite corpus proves an absolute claim about every program, machine or future
upstream revision. The acceptance threshold remains `1.0` with no regression
allowance.

### Summary, evidence and publication

[`tools/native-ci-summary.py`](../../tools/native-ci-summary.py) always renders
the available report into the GitHub job summary and an uploaded Markdown file.
It includes source and compiler identities, build settings, workflow outcomes,
every required timing mode, compatibility requirements, paired ratios,
confidence bounds, worst pairs and execution failures. Missing rows appear as
unproved. Missing or malformed report files produce an unqualified summary and
a failing renderer exit. A qualified headline also requires matching source
identities, the current release receipt recipe and successful recorded workflow
checks. The workflow passes `--enforce`, so inconsistent or missing corpus
evidence, source mismatches and different compiler hashes fail before archive
creation. The summary does not replace the executable or performance gates,
and never upgrades their result. Without `--enforce`, it can also inspect a
historical compressed report locally:

```sh
python3 tools/native-ci-summary.py \
  --report docs/compiler-set/measurements/2026-11-11-native-composition/optimized-ci/native-speed-evidence/report.json.gz \
  --output /tmp/lustr-native-summary.md
```

Evidence uploads run after failures. Archive creation and installable package
upload require successful prior checks, including strict native acceptance.
An accepted individual timing mode never overrides incomplete Rust coverage or
another failed mode. This workflow does not change repository merge rules or
automatically merge a draft PR.

## Upstream designs reviewed

These references are pinned to the source inspected on 2026-11-11. They inform
the workflow structure; their thresholds and coverage claims are not inherited.

| Source | Observed mechanism | Applied lesson |
| --- | --- | --- |
| [Lust Cargo profiles](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/Cargo.toml) and [build helper](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/build.sh) | Gate builds support fast correctness iteration; release builds support performance measurement. A helper can copy a gate binary into a release directory. | Bind actual compiler flags and binary hashes; never infer a build profile from its pathname. |
| [Lust warm checks](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/warm-check.sh) and [speed row](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/speed-row.sh) | Warm checks enumerate cases and reject missing results. The speed row is explicitly non-blocking. | Expose every required mode, including absent evidence. Lustr's speed requirement remains blocking. |
| [t3-code CI](https://github.com/pingdotgg/t3code/blob/3ed5fb29bd2ffdb31cf146f016300e26ca237f99/.github/workflows/ci.yml) and [change detection](https://github.com/pingdotgg/t3code/blob/3ed5fb29bd2ffdb31cf146f016300e26ca237f99/.github/actions/ci-changes/action.yml) | Changed paths select affected jobs, full default-branch runs provide a backstop, and the final check requires justified skips and measurement artifacts. Unknown diffs run every job. | Scope the expensive unchanged-donor workflow, keep full manual/scheduled backstops, and preserve missing evidence as nonpassing. Lustr does not use t3-code's hosted runner provider. |
| [t3-code dependency setup](https://github.com/pingdotgg/t3code/blob/3ed5fb29bd2ffdb31cf146f016300e26ca237f99/.github/actions/setup-ci-deps/action.yml) | Cache keys bind tool versions, architecture and dependency metadata. Filtered dependency sets have separate namespaces. | Cache locked downloads with explicit compiler lockfiles and host identity; do not replay compiler outputs or qualification receipts across source changes. |
| [rustc-perf collector](https://github.com/rust-lang/rustc-perf/blob/1aa248c66e940d68d58b45e9b48430f6c4489797/collector/README.md) | It distinguishes full, incremental-full, unchanged and patched compilation, supports an explicit Cargo executable, and records several metrics. Instruction counts are less variable than wall time. | Separate cold, no-op and edit modes, pin Cargo, and retain noise/uncertainty. Lustr's required wall-time speed floor cannot be replaced by an instruction-count improvement. |
| [Rust CI](https://github.com/rust-lang/rust/blob/e88737b98536eae309f98e207b8af6a2cafa0350/.github/workflows/ci.yml) and [rustc-perf CI](https://github.com/rust-lang/rustc-perf/blob/1aa248c66e940d68d58b45e9b48430f6c4489797/.github/workflows/ci.yml) | Rust generates jobs from committed definitions and reports build metrics. rustc-perf has a summary job that explicitly checks dependency results. | Keep the source of required modes in executable code and expose all job failures; do not treat a skipped check as proof of success. |
| [LLVM interleaved A/B workflow](https://github.com/llvm/llvm-project/blob/0d43c90fb79c2eb1f55a56eaee9a021fe4ecce1f/.github/workflows/libcxx-run-ab-benchmarks.yml) | Baseline, candidate and benchmark suite are pinned independently. Builds happen once, samples run serially on the same machine, and raw evidence is retained before comparison results. | Preserve one build and a sequential paired campaign, exact tool/input identities, complete artifacts and an immediately readable job summary. |

Upstream updates require an explicit pin or toolchain change and a fresh
qualification. CI does not silently replace the baseline with a moving branch,
reuse a previous passing decision, or relabel the current unsupported Rust
fixtures as optional.
