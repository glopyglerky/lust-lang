# Public repository checks, 2026-11-09

Four frontend threads improved BlueFe's measured root-check time relative to one
thread on all three workloads. Against stock Rust, four threads improved only
`t3code-ls`. The point estimates regressed for `gitr` and `bluefin`. Explicit
one-thread BlueFe regressed against stock in every case. These results do not
support a general claim that Lustr/BlueFe is faster than stock Cargo/rustc.

This is a shared-host, dependency-primed, metadata-only root-check experiment.
It measures neither a default whole-project build nor cold disk-cache compilation.
The frozen preview compiler and coordinator were used without rebuilding them
for each project. Full Rust compilation still comes from the rustc compiler family.

## Results

Ratios are geometric means of paired baseline/candidate wall times. Above 1
favors the denominator lane. Brackets are conditional 95% percentile bootstrap
intervals. `N1` and `N4` mean explicit BlueFe frontend options 1 and 4, with the
same compiler and a Cargo job budget of 4.

| Project | Stock/N1 | Stock/N4 | N1/N4 |
| --- | --- | --- | --- |
| t3code-ls | 0.541 [0.522, 0.563] | 1.268 [1.209, 1.341] | 2.345 [2.241, 2.464] |
| gitr | 0.690 [0.667, 0.715] | 0.927 [0.875, 0.990] | 1.342 [1.287, 1.419] |
| bluefin | 0.512 [0.468, 0.561] | 0.923 [0.726, 1.159] | 1.802 [1.501, 2.140] |

For t3code-ls, Stock/N4 = 1.268 corresponds to about 21% less paired geometric
wall time. For gitr and bluefin, N4 point estimates took about 8% more time than
stock. The bluefin interval crosses parity. N1/N4 is a same-compiler scheduling
control, not evidence from a second independent Rust compiler implementation.

| Project | Stock median seconds | N1 median seconds | N4 median seconds | Sign-flip p, Stock/N1; Stock/N4; N1/N4 |
| --- | --- | --- | --- | --- |
| t3code-ls | 0.656 | 1.147 | 0.521 | 0.0078125; 0.0078125; 0.0078125 |
| gitr | 0.251 | 0.372 | 0.283 | 0.0078125; 0.0625; 0.0078125 |
| bluefin | 6.862 | 14.610 | 7.044 | 0.0078125; 0.5703125; 0.0078125 |

There are eight paired rounds per comparison. Nine comparisons share the same
72 timed observations. Nine additional warmup observations were excluded as
registered. All 204 command records, including setup and qualification, are in
the sanitized dataset. No slow sample or regression was removed. With eight
samples, nearest-rank p95 is just the maximum and supports no tail-latency claim.

## Workloads and tools

| Repository revision | Timed package and target | Project test gate per lane, before and after |
| --- | --- | --- |
| [t3code-ls b1601816](https://github.com/glopyglerky/t3code-ls/commit/b16018167df26a18ee75d3d2b8f4bf2fcd64ce92) | `t3-code-ls-core --lib` | 56 passed |
| [gitr a183e205](https://github.com/glopyglerky/gitr/commit/a183e205ec21e1b916e3763ba85917f2842aeae5) | `gitr --bin gitr` | Library tests: 38 passed, 5 project-ignored |
| [bluefin b7d124a2](https://github.com/glopyglerky/bluefin/commit/b7d124a22d49bacf19f6c3c940333f508d647fb6) | `ferrofin-model --lib` | 628 passed |

The common Cargo tail was `rustc --profile check -p PACKAGE TARGET_SELECTOR
--locked --offline --target x86_64-unknown-linux-gnu -vv`. The root timestamp was
changed without changing bytes; dependencies were primed. Project defaults,
features and locked dependency selection were preserved. Incremental compilation
was disabled. Observed root invocations used edition 2024, debuginfo 2,
`--emit=dep-info,metadata`, and the requested BlueFe frontend option. Those
arguments show the requested scheduling setting, not actual thread utilization.

Stock was rustc 1.99.1, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`,
LLVM 23.1.1, Cargo 1.99.1 commit `5f94df4789f005f9a352888e8355ffc645b7ed0e`.
BlueFe was rustc 1.102.0-dev, commit
`1b74388e196ee8813f38ba9eeb6d4ab43b6b1071`, LLVM 23.1.5, with upstream
bootstrap Cargo commit `31d3246e0e8d3a80d4a90b54aa48ff180adb408b`.
Lustr coordinator source was `07470b20464bf097c3c88983124aff56c5e44f9a`.
The [published preview](https://github.com/glopyglerky/lustr/releases/tag/preview-2026.11.09-1)
contains those components. Different compiler, LLVM and Cargo versions confound
the stock contrast. The benchmark runner source is pinned to
[`3f3e4c48`](https://github.com/glopyglerky/lustr/tree/3f3e4c480c55a97de007894b4b91fc9519595019/crates/lustr-bench).
Exact executable and original report hashes accompany the samples.

The host was Intel Core Ultra 5 235T, Linux 7.2.7-63.stable, with allowed CPUs
10-13 and at least 16 GiB available memory at admission. Existing services and
competing work continued. Every command retains before/after CPU counters,
load and CPU/I/O pressure. These are bracket observations, not isolation,
continuous resource accounting or thermal control.

## Qualification and limits

Source bytes and selected dependency-source observations matched before and
after. Root compiler evidence, nonempty Cargo fingerprints and full metadata
selection checks passed. Only the versioned absent/null `hints.min-opt-level`
[projection](../../proofs/metadata-compatibility.md) was applied: stock had none,
while each BlueFe lane had two projections for t3code-ls and one for each other
case. Non-null values and unknown fields were not erased.

Direct selected-tool invocations passed the project tests above, a native
panic-recovery fixture with exactly one Drop, and structured E0502 rejection.
These gates **did not pass the explicit N1/N4 frontend options** used by the
timed checks. They qualify those direct toolchain invocations, not native runtime
equivalence under both scheduling modes or interchangeability of their metadata.
No timing conclusion is a proof of semantic compiler equivalence.

The order alternated registered forward/reverse sequences. **N1 always ran in
the middle position.** Stock and N4 alternated first and last. Position and cache
history therefore remain confounders, including in the N1/N4 control. The same
seeded sequence was used for all cases. A future crossover must balance each
lane's positions and qualify the actual scheduling options before timing.

Bootstrap intervals assume representative, independent pairs. Sign-flip p-values
assume sign exchangeability under the null. Neither assumption is established
by passing gates or observed host load. All nine correlated exploratory tests
are unadjusted for multiplicity. A conditional 95% bootstrap interval is not a
95% probability of improvement or a simultaneous coverage guarantee. Gitr's
Stock/N4 interval excludes 1 while its sign-flip p is 0.0625; these are distinct
procedures, not interchangeable significance guarantees.

RSS, energy, cold-cache builds, ordinary whole-project build time, edits,
rejection latency, compile-plus-test timing, Cranelift and Wasm execution were
not measured here. Qualification and hashing outside the timer can affect
caches. The earlier quiet-host campaign remained blocked with null conclusions;
these are separately registered shared-host results.

## Mathematical scope and replay

For paired seconds `a_i, b_i > 0`, define
`G(A,B) = exp(sum_i(log(a_i) - log(b_i)) / n)`. For the same paired observations,
subtracting logs gives `G(A,C) = G(A,B) * G(B,C)` in exact real arithmetic.
Therefore the three ratios in each row share information. Their confidence
intervals cannot be multiplied to obtain another valid confidence interval.
The [analysis](../../proofs/benchmark-analysis.md) derives estimator invariants,
bootstrap assumptions and sign-flip limits; the
[frontend proof](../../proofs/frontend-scheduling.md) covers argument preservation
and work/span bounds. These are scoped mathematical derivations and executable
checks, not proof-assistant verification or proofs of rustc correctness or speed.

[`samples.json`](samples.json) retains all command records with path-free fields,
full-precision paired seconds, original order, excluded warmups, output hashes,
host observations, tool identities and all nine summaries.
[`observations.csv`](observations.csv) is the 81-row timing/warmup view.
Absolute paths, raw output, argv, environment paths and source inventories remain
private. The export is an allowlisted derivative, not the original report or a
complete public re-execution closure. Its hashes differ from the bound private
report hashes. The N1/N4 method label, medians and p95 values were added from
the same timed records to complete the Rust summary schema; its original
numerical fields are preserved. [`SHA256SUMS`](SHA256SUMS) binds the three
report files in this directory.

Recalculate all nine summaries using the runner's Rust statistics module:

```text
cargo run --locked --release --manifest-path crates/lustr-bench/Cargo.toml \
  --example replay -- docs/compiler-set/measurements/2026-11-09-shared-v2/samples.json
```

This reads data and prints JSON; it runs no compiler workloads. It verifies pair
cardinality, unique lanes/comparisons, successful timed records and every stored
summary field, allowing 1e-12 relative-or-absolute floating rounding. Replaying
statistics does not authenticate the private raw measurements or qualify a
compiler. Independent review reproduced all nine original estimates, bootstrap
endpoints and sign-flip results before public export.

The replay example built with the produced preview compiler and reproduced all
nine exported summaries. Altering a summary or inserting a duplicate timed
observation caused rejection. Matching rustfmt and Clippy with warnings denied
passed. The benchmark runner itself was not changed for this export.

## Next experiments

Keep frontend parallelism explicit. The evidence does not justify a global
four-thread default. Next, build a fresh stage2 control before compiler PGO,
then compare a separately trained PGO compiler on held-out workloads. CGU 16
plus ThinLTO is a separate configuration experiment. A rebuild's source, stage,
optimization, profile, feature, target, backend and native-library inputs must
remain recorded; stage1 versus stage2 alone cannot attribute a gain to PGO.
No result from those unexecuted comparisons is included here.
