# Native cache work: release baseline

This run precedes the native cache implementation. It is **unqualified**.
Both compilers used their actual release recipe. All standalone correctness
steps, package relocation and bounded concurrent Cargo checks passed. Strict acceptance failed,
so no installable archive was created.

Source: [`8110d9f7aad82a8c382ebadcc092248f0db917bc`](https://github.com/glopyglerky/lustr/commit/8110d9f7aad82a8c382ebadcc092248f0db917bc).
[CI run](https://github.com/glopyglerky/lustr/actions/runs/43069040645),
[complete artifact](https://github.com/glopyglerky/lustr/actions/runs/43069040645/artifacts/13876854325).
The artifact ZIP SHA-256 is
`299eeaa25cff67c65f9ddc94d19d43b8f8cb2b6556e18cf6d1bdfc9e86e6369e`.

## Measurements

All 51 required timing series are retained in [timing-summary.csv](timing-summary.csv).
There are 18 passed, 20 failed and 13 unproved series. Each measured series has
12 alternating pairs. The table covers every measured compile-only series.
Medians are milliseconds. Ratios are geometric means of paired candidate /
baseline durations, not ratios of the displayed medians. Lust is the baseline
for `lust/`; the same Cargo/rustc is the baseline for `cargo/` and `mixed/`.

| Mode | Baseline ms | Native ms | Paired ratio | Slower pairs | State |
| --- | ---: | ---: | ---: | ---: | --- |
| mixed/fresh-target | 83.152 | 83.399 | 0.990 | 6/12 | failed |
| mixed/no-change | 28.359 | 16.636 | 0.580 | 0/12 | passed |
| mixed/body-edit | 65.324 | 49.911 | 0.748 | 0/12 | passed |
| mixed/signature-edit | 64.574 | 48.948 | 0.746 | 0/12 | passed |
| mixed/type-edit | 67.489 | 49.254 | 0.768 | 1/12 | failed |
| mixed/macro-edit | 70.137 | 49.748 | 0.696 | 0/12 | passed |
| lust/process-cold | 43.706 | 43.672 | 0.955 | 4/12 | failed |
| lust/warm-noop | 4.794 | 24.265 | 4.868 | 12/12 | failed |
| lust/body-edit | 6.159 | 26.989 | 3.217 | 11/12 | failed |
| lust/signature-edit | 12.148 | 24.861 | 1.959 | 12/12 | failed |
| lust/type-edit | 11.519 | 24.037 | 1.969 | 12/12 | failed |
| lust/macro-edit | 6.818 | 26.220 | 3.866 | 12/12 | failed |
| lust/cold-invalid | 6.389 | 6.042 | 0.930 | 3/12 | failed |
| cargo/fresh-target | 81.715 | 66.087 | 0.783 | 0/12 | passed |
| cargo/no-change | 37.976 | 18.410 | 0.506 | 0/12 | passed |
| cargo/body-edit | 82.250 | 45.786 | 0.550 | 0/12 | passed |
| cargo/signature-edit | 80.595 | 45.428 | 0.560 | 0/12 | passed |
| cargo/type-edit | 80.478 | 45.800 | 0.565 | 0/12 | passed |
| cargo/macro-edit | 82.240 | 49.135 | 0.731 | 2/12 | failed |
| cargo/cold-invalid | 56.174 | 25.570 | 0.450 | 0/12 | passed |

The direct cold medians are nearly equal, while retained-daemon comparisons
still expose slower native warm work. Mixed cold medians are also nearly equal;
its strict gate fails on slower observations. Do not transfer changes between
the earlier gate-profile run and this release run into a controlled performance
claim: they are different builds and separate CI executions.

The new Cargo and mixed type/macro modes each completed 12 real paired
observations. Both engines produced the required changed runtime output and
reported one exact source compiler invocation for each edit. The new full-Rust
type/macro modes were attempted too, but their native warmups failed.

All eight full-Rust modes fail native compilation at `cannot produce proc-macro`
for `fixture-derive`; the unchanged stock reference passes each preflight.
The complete upstream accepted-input corpus remains unproved. Native full-Rust
compatibility fails, while native build-script and doctest coverage remain
unproved. No failed compilation is counted as a fast compilation.

## Evidence and verification

- [speed-report.json.gz](speed-report.json.gz) and [cargo-report.json.gz](cargo-report.json.gz) restore the exact original report bytes.
- [native-cargo-build.json](native-cargo-build.json) and [native-cargo-build.cargo.log.gz](native-cargo-build.cargo.log.gz) retain the v2 release receipt and its observed bootstrap command.
- [native-qualification-summary.md](native-qualification-summary.md) is the actual CI summary; [workflow-jobs.json](workflow-jobs.json) records all workflow outcomes.
- [raw-evidence.tar.gz](raw-evidence.tar.gz) retains every speed-command stdout/stderr, both compiler traces, all generated edited inputs, direct sources and the five upstream daemon logs. Archive paths retain their original `native-speed-evidence/` prefix.
- [timing-pairs.csv](timing-pairs.csv) retains all 456 reported observations. Compile-plus-run rows reuse build durations and add runtime, so these are not 456 independent build trials.
- [verification.json](verification.json) records 676 commands, all 1,352 matching raw output hashes, 419 inherited Cargo commands, matching source/helper/build identities, exact edit rebuilds and genuine compiler overlap.
- [manifest.json](manifest.json) binds original and compressed bytes, tool identities, source, build settings, artifact identity and expiry. The full GitHub artifact expires on 2027-02-09; this directory keeps the measurement evidence independently.

Compiler binaries are identified by the CI receipts and reports. They are not
included in this text-evidence subset. The matched native release recipe is
opt3, one codegen unit, ThinLTO, line-table debug information and no incremental
compiler build. The unchanged upstream source tree is
`0ab1b09815802894e6078de0af59273cbdf46cb0`.

To inspect the records without replacing them:

```sh
gzip -dc speed-report.json.gz > /tmp/lustr-release-baseline.json
gzip -dc cargo-report.json.gz > /tmp/lustr-release-baseline-cargo.json
tar -tzf raw-evidence.tar.gz
```

This evidence describes a finite corpus on one shared CI host. It establishes
neither complete Rust coverage nor universal cold or warm speed parity.
