# Initial native request-cache candidate

This candidate is **unqualified**. Every independent workflow correctness step
passed, including the published cache gate's 75 checks. Strict performance and
full-Rust acceptance failed. No installable archive was produced.

The measured source is [`5b98ffb63da0139d778ef4a26de28b1d38fd4a0e`](https://github.com/glopyglerky/lustr/commit/5b98ffb63da0139d778ef4a26de28b1d38fd4a0e).
[CI run](https://github.com/glopyglerky/lustr/actions/runs/43072307206),
[complete artifact](https://github.com/glopyglerky/lustr/actions/runs/43072307206/artifacts/13882595194).
ZIP SHA-256: `05ad07f90a50b0b36a03366342628eefa80f088748dadb34fd4db3be5e457d18`.

This run predates the separately discovered parent-`Cargo.toml` cache
correctness fix. Its 75 published checks do not cover that case. These timings
must not be attributed to a later revision containing that fix.

## Measurements

All 53 required series are retained: 11 passed, 29 failed and 13 unproved.
Each measured series has 12 alternating pairs. Medians below are milliseconds;
ratios are geometric means of paired candidate/baseline durations, not ratios
of the displayed medians. Lust is the same-run reference for `lust/`; stock
Cargo/rustc is the reference for `cargo/` and `mixed/`.

| Mode | Baseline ms | Native ms | Paired ratio | Slower pairs | State |
| --- | ---: | ---: | ---: | ---: | --- |
| lust/process-cold | 50.116 | 50.840 | 1.006 | 8/12 | failed |
| lust/fresh-cache | 51.396 | 52.408 | 1.010 | 7/12 | failed |
| lust/warm-noop | 4.681 | 4.407 | 0.960 | 4/12 | failed |
| lust/body-edit | 6.339 | 28.869 | 4.542 | 12/12 | failed |
| lust/signature-edit | 12.864 | 28.217 | 1.861 | 11/12 | failed |
| lust/type-edit | 12.897 | 28.145 | 2.280 | 12/12 | failed |
| lust/macro-edit | 6.959 | 28.289 | 4.034 | 12/12 | failed |
| lust/cold-invalid | 6.276 | 5.602 | 0.933 | 3/12 | failed |
| cargo/fresh-target | 81.878 | 75.252 | 0.917 | 1/12 | failed |
| cargo/no-change | 37.129 | 19.142 | 0.511 | 0/12 | passed |
| cargo/body-edit | 80.225 | 50.709 | 0.628 | 0/12 | passed |
| cargo/signature-edit | 80.960 | 49.617 | 0.573 | 1/12 | failed |
| cargo/type-edit | 81.661 | 50.312 | 0.733 | 1/12 | failed |
| cargo/macro-edit | 81.249 | 49.823 | 0.615 | 0/12 | passed |
| cargo/cold-invalid | 54.516 | 26.423 | 0.480 | 0/12 | passed |
| mixed/fresh-target | 83.050 | 88.945 | 1.078 | 12/12 | failed |
| mixed/no-change | 28.114 | 16.993 | 0.672 | 1/12 | failed |
| mixed/body-edit | 67.034 | 53.621 | 0.848 | 1/12 | failed |
| mixed/signature-edit | 67.360 | 52.425 | 0.779 | 0/12 | passed |
| mixed/type-edit | 68.380 | 52.494 | 0.814 | 2/12 | failed |
| mixed/macro-edit | 67.385 | 53.079 | 0.780 | 0/12 | passed |

The warm-noop median is close to Lust, but four pairs are slower. Both lanes
contain observations around 119 ms. The worst paired ratio is 24.909, and its
one-sided 95% upper ratio is 1.680. All observations remain in the gate; none is
removed as noise. This run does not identify the cause of those long samples.
Process-cold and enabled-cache first admission also fail the zero-regression
requirement. Body, signature, type and macro edits remain slower than Lust.

Fresh-cache samples used 24 distinct empty compilation caches and 24 distinct
empty runtime-cache directories across the two lanes. Caching was enabled;
daemon reuse was disabled. Recorded actual command environments match those
paths. Each Cargo/mixed type and macro edit produced real rebuilt output with
exactly one source compiler invocation in each lane.

All eight full-Rust native preflights failed at `cannot produce proc-macro` for
`fixture-derive`; each unchanged stock reference passed. These failures took
19–25 ms, without a timeout or Cargo lock stall. Complete upstream accepted-input
coverage, native build scripts and doctests remain unproved. Full-Rust
compatibility remains failed. Failed compilations are not timing passes.

## Retained evidence

- [speed-report.json.gz](speed-report.json.gz), [cargo-report.json.gz](cargo-report.json.gz) and [request-cache-report.json.gz](request-cache-report.json.gz) restore the original report bytes.
- [native-cargo-build.json](native-cargo-build.json) and [native-cargo-build.cargo.log.gz](native-cargo-build.cargo.log.gz) retain the release v2 receipt and actual bootstrap command.
- [native-qualification-summary.md](native-qualification-summary.md) is the original CI summary. [workflow-jobs.json](workflow-jobs.json), [workflow-job.log.gz](workflow-job.log.gz) and [artifact-metadata.json](artifact-metadata.json) retain the GitHub API evidence.
- [raw-evidence.tar.gz](raw-evidence.tar.gz) retains every speed/cache-check stdout and stderr, both compiler traces, generated edited inputs, direct sources and daemon logs, with original relative paths.
- [timing-summary.csv](timing-summary.csv) retains all 53 series. [timing-pairs.csv](timing-pairs.csv) retains all 480 reported observations; compile-plus-run entries reuse compilation durations and are not independent build trials.
- [verification.json](verification.json) records raw-output checks, source/helper identities, first-admission cache isolation, edit rebuilds and actual compiler concurrency. [manifest.json](manifest.json) binds retained bytes, source, compiler identities, settings and artifact expiry.

Both native and Lust builds used their actual release recipe. Native settings
are opt3, one codegen unit, ThinLTO, line-table debug information and incremental
disabled. The upstream native tree is unchanged. Compiler executables are
identified by CI receipts; their bytes are absent from this text subset.

The preceding release baseline is a separate CI execution with different host
conditions. Do not treat changes between those runs as a controlled speedup;
each run's native/Lust comparison is the relevant paired evidence.
