# Optimized native composition CI, 2026-11-11

This optimized run is **unqualified**. Full Rust compatibility and the required
speed floors remain unmet. The strict gate failed, so the workflow skipped the
installable package archive and preview upload. It uploaded diagnostic evidence.

## Source and build

The measured source is
[`c3d57f630cb4d2909e9217ae0fa074225376f76a`](https://github.com/glopyglerky/lustr/commit/c3d57f630cb4d2909e9217ae0fa074225376f76a).
[Workflow run 43063185428](https://github.com/glopyglerky/lustr/actions/runs/43063185428)
produced [evidence artifact 13876190619](https://github.com/glopyglerky/lustr/actions/runs/43063185428/artifacts/13876190619).
The downloaded ZIP is 92,467,266 bytes with SHA256
`8aca8c0ad9508773fab604e40f11c7092e8892b33af03c484e50b224c58c246f`.

The exact [native build receipt](native-cargo-build.json) and its
[bound Cargo build log](native-cargo-build.cargo.log) record Rust 1.99.1, the
`gate` profile, optimization level 3, 16 codegen units, ThinLTO, debug information
disabled and incremental compilation enabled. This is separate from the earlier
optimization-level-1 diagnostic record in the parent directory. The baseline
upstream Lust used its release profile. These measurements compare the recorded
compiler builds, without isolating the cost of any individual code change.

## Observed results

| Requirement or workflow stage | Result | Scope |
| --- | --- | --- |
| Source protection and build provenance | PASS | Pinned upstream source identity and exact native build receipt |
| Correctness gates before strict acceptance | PASS | Parser, artifact, protocol, unsafe, mixed generic and safe-library checks |
| Relocation and independent native library qualification | PASS | Required bounded native library interfaces and source-hidden consumers |
| Mixed Cargo compatibility and concurrency | PASS | The recorded two-library Cargo graph, with two compiler jobs |
| Complete upstream accepted-input corpus | UNPROVED | Representative direct fixtures do not cover the complete corpus |
| Full Rust compatibility | FAIL | All six candidate commands stop at unsupported proc-macro output |
| Strict speed acceptance | FAIL | 11 series passed, 19 failed and 21 remained unproved |
| Installable archive and preview upload | SKIPPED | Strict acceptance failed before these steps |

The inherited Cargo report's `success: true` applies to its dependency-free
native binary fixture and supported profile checks. It explicitly records
`native_full_rust: false` and `full_native_speed_qualified: false`.

## Recorded timing

Each measured series contains 12 pairs, with alternating baseline/candidate
order, no trace during timing, no discarded observations and zero regression
allowance. Passing requires every observed pair to be non-regressing and the
one-sided 95% paired bootstrap upper bound to be at most 1. A faster median
alone does not pass this policy.

The table shows every measured compile-only series. Medians are milliseconds;
the ratio is the geometric mean of paired native/baseline ratios, not the ratio
of the two medians. `lust/` uses upstream Lust as baseline. `cargo/` and `mixed/`
use the same stock Cargo/rustc toolchain as baseline.

| Series | Baseline ms | Native ms | Ratio | Slower pairs | Gate |
| --- | ---: | ---: | ---: | ---: | --- |
| lust/process-cold | 49.916 | 49.866 | 1.003 | 4/12 | FAIL |
| lust/warm-noop | 4.846 | 27.851 | 5.682 | 12/12 | FAIL |
| lust/body-edit | 6.374 | 28.493 | 4.400 | 12/12 | FAIL |
| lust/signature-edit | 12.792 | 26.640 | 2.038 | 12/12 | FAIL |
| lust/type-edit | 12.672 | 27.449 | 2.365 | 12/12 | FAIL |
| lust/macro-edit | 7.291 | 29.168 | 3.938 | 12/12 | FAIL |
| lust/cold-invalid | 6.864 | 6.395 | 0.932 | 2/12 | FAIL |
| cargo/fresh-target | 81.767 | 75.157 | 0.961 | 1/12 | FAIL |
| cargo/no-change | 38.743 | 19.815 | 0.512 | 0/12 | PASS |
| cargo/body-edit | 84.353 | 51.649 | 0.609 | 0/12 | PASS |
| cargo/signature-edit | 84.007 | 51.688 | 0.787 | 2/12 | FAIL |
| cargo/cold-invalid | 57.567 | 26.693 | 0.468 | 0/12 | PASS |
| mixed/fresh-target | 84.672 | 92.052 | 1.100 | 12/12 | FAIL |
| mixed/no-change | 28.942 | 17.499 | 0.604 | 0/12 | PASS |
| mixed/body-edit | 67.917 | 53.076 | 0.786 | 0/12 | PASS |
| mixed/signature-edit | 68.574 | 52.651 | 0.772 | 0/12 | PASS |

The five warm/edit comparisons against upstream Lust regressed in every pair.
The full artifact retains five valid upstream daemon logs. Cold direct results
were effectively tied and noisy; their four slower observations fail the strict
gate but do not establish a systematic cold regression. Mixed cold builds
regressed in all 12 pairs, with a geometric ratio of 1.100.

[timing-summary.csv](timing-summary.csv) retains all 51 series, including
compile-plus-run, failed and unproved rows, their upper bounds and failure
reasons. [timing-pairs.csv](timing-pairs.csv) retains every raw duration and ratio
from the 30 measured series. Its 360 rows include compile-plus-run observations
that reuse the corresponding build measurement and add execution time; they are
not 360 independent build trials. These results describe this finite corpus,
host, compiler identities and settings. They prove no universal speed claim.

## Full Rust failures

All six native commands exited 101 because Cargo could not produce the
`fixture-derive` proc-macro crate with the selected native compiler. The stock
baseline accepted the unchanged workspace and exited 0 in every mode. This is
the native compiler's missing crate-type support; the baseline successfully
uses the same host target. The test and doctest commands stopped before their
native test semantics could be assessed. Native build-script and doctest
support remain unproved by this run, and failed compilation durations do not
count as speed results.

| Mode | Native failure stderr | Baseline command output | Baseline executable output |
| --- | --- | --- | --- |
| Fresh target | [213](native-speed-evidence/logs/00213.stderr) | [stdout](native-speed-evidence/logs/00214.stdout), [stderr](native-speed-evidence/logs/00214.stderr) | [stdout](native-speed-evidence/logs/00215.stdout), [stderr](native-speed-evidence/logs/00215.stderr) |
| No change | [216](native-speed-evidence/logs/00216.stderr) | [stdout](native-speed-evidence/logs/00217.stdout), [stderr](native-speed-evidence/logs/00217.stderr) | [stdout](native-speed-evidence/logs/00218.stdout), [stderr](native-speed-evidence/logs/00218.stderr) |
| Check | [219](native-speed-evidence/logs/00219.stderr) | [stdout](native-speed-evidence/logs/00220.stdout), [stderr](native-speed-evidence/logs/00220.stderr) | Not applicable |
| Release | [221](native-speed-evidence/logs/00221.stderr) | [stdout](native-speed-evidence/logs/00222.stdout), [stderr](native-speed-evidence/logs/00222.stderr) | [stdout](native-speed-evidence/logs/00223.stdout), [stderr](native-speed-evidence/logs/00223.stderr) |
| Compile plus test | [224](native-speed-evidence/logs/00224.stderr) | [stdout](native-speed-evidence/logs/00225.stdout), [stderr](native-speed-evidence/logs/00225.stderr) | Included in command output |
| Doctests | [226](native-speed-evidence/logs/00226.stderr) | [stdout](native-speed-evidence/logs/00227.stdout), [stderr](native-speed-evidence/logs/00227.stderr) | Included in command output |

These are functional preflight commands, identified by sample `-1` in the full
report. The baseline executable stderr includes the fixture's caught-panic
diagnostic; the executable still exits 0 with the expected output.

## Concurrency evidence

The [native trace](native-speed-evidence/logs/00005.trace) shows successful
compilation of `left/src/lib.rs` and `right/src/lib.rs` by native compiler PIDs
9371 and 9372, overlapping by 38.890 ms. The
[stock trace](native-speed-evidence/logs/00007.trace) shows the corresponding
rustc processes overlapping by 66.614 ms. These traces establish actual Cargo
parallelism for the bounded fixture. They are functional checks excluded from
the timing samples, and do not establish full Rust composition.

## Retained evidence

The complete [strict speed report](native-speed-evidence/report.json.gz) and
[inherited Cargo report](native-speed-evidence/existing-cargo-qualification.json.gz)
are losslessly compressed with gzip timestamp zero and an empty filename.
Decompression restores their exact original bytes, including every command,
sample, failure and runner path. [manifest.json](manifest.json) records both
the original and compressed SHA256 values, file sizes, artifact identity and
the hashes of all retained raw files and derived CSVs.

From this directory, inspect the reports without replacing the compressed files:

```sh
gzip -dc native-speed-evidence/report.json.gz > /tmp/lustr-optimized-speed-report.json
gzip -dc native-speed-evidence/existing-cargo-qualification.json.gz > /tmp/lustr-optimized-cargo-report.json
sha256sum /tmp/lustr-optimized-speed-report.json /tmp/lustr-optimized-cargo-report.json
```

This directory retains the relevant failure/success logs, concurrency traces
and build provenance. The full artifact retains the remaining generated
outputs, daemon logs, qualification reports and binaries. GitHub records its
expiry as **2027-02-09 16:56:09 UTC**. Some unchanged report paths therefore
refer to files available only in that full artifact. No installable package
archive was produced by this run.
