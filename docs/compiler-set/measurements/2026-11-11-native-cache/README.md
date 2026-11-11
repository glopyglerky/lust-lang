# Native request-cache measurements

These records test the bounded unchanged-request cache. They do **not** qualify
the compiler against the cold or warm speed floors, or establish full Rust
coverage. The source parent is
[`8110d9f7`](https://github.com/glopyglerky/lustr/commit/8110d9f7aad82a8c382ebadcc092248f0db917bc).
The [performance design](../../native-performance.md) records the implementation
and upstream evidence.

The [release baseline](release-baseline/README.md) retains the completed
pre-cache CI run, its exact v2 build receipt, all 51 then-required series and
verified raw output. It is unqualified. The candidate adds two required
first-admission series, bringing the current total to 53.

## Correctness

The final diagnostic candidate passed all **75 checks in 58 commands** in
[`functional.json.gz`](functional.json.gz). The test disables a private TinyCC
helper after compilation, so a successful unchanged request demonstrates actual
reuse. It also checks dependency information, Cargo notifications, changed
inputs, compiler replacement, umask, corruption and output failures. All 116
stdout/stderr hashes match the retained
[`functional-logs.tar.gz`](functional-logs.tar.gz).

The earlier candidate passed the independent source-race and output-failure
review, **10 checks in 6 commands**, retained in
[`independent-review.json.gz`](independent-review.json.gz). A source edited after
frontend consumption did not enter the cache. A dependency-info failure during
a proven cache hit removed the executable and emitted no successful artifact
notification. The reusable [review runner](independent-review-runner.py) was
saved after that inline run, then replayed against the exact same binary.
All ten checks passed again in [the replay](independent-review-replay.json.gz).
The same candidate passed the existing library regression's
13 required checks in 125 commands, retained in
[`library-functional.json.gz`](library-functional.json.gz). Process tracing was
unavailable locally, so this diagnostic library run is not qualification.

## Diagnostic timing

Both candidates and the reference below used optimization level 1 with ThinLTO
disabled. They are not the release builds required for acceptance. Each series
retains 12 interleaved pairs, including slower observations. These are separate
before/after experiments on this executor, not comparisons against Lust or
stock rustc. Program execution is excluded from these compile timings.

The first experiment used the candidate before the final lazy formatting change.
Warm requests remove the output and dep-info and retain the cache. Its cold
requests explicitly set `LUST_NO_CACHE=1` and use a new empty runtime temporary
directory. Consequently those cold observations do not measure cache admission.
The raw record is [`paired-diagnostic.json.gz`](paired-diagnostic.json.gz).

| Workload | Uncached cold reference / candidate, ms | Warm reference / candidate, ms | Slower warm pairs |
| --- | ---: | ---: | ---: |
| Generics and traits | 51.96 / 51.10 | 29.59 / 5.60 | 2/12 |
| Unsafe scalar | 56.27 / 54.15 | 31.60 / 5.38 | 0/12 |
| Unsafe generic | 50.63 / 51.36 | 29.55 / 4.22 | 0/12 |
| Many functions | 50.06 / 50.51 | 29.69 / 4.62 | 0/12 |

The second experiment used the final candidate and enabled caching. Every
sample started with empty artifact/unit and runtime directories and removed
the output and dep-info. Every candidate invocation created one admission,
including each preflight. This measures first-use lookup and store cost. All
four candidate medians were slower. The raw record is
[`fresh-cache-diagnostic.json.gz`](fresh-cache-diagnostic.json.gz).

| Workload | Reference median, ms | Candidate median, ms | Slower pairs |
| --- | ---: | ---: | ---: |
| Generics and traits | 51.71 | 56.51 | 5/12 |
| Unsafe scalar | 49.02 | 56.90 | 7/12 |
| Unsafe generic | 55.66 | 56.12 | 6/12 |
| Many functions | 55.98 | 57.83 | 8/12 |

The warm improvements do not offset cold regressions. Release comparisons must
include first admission as well as disabled-cache cold compilation, unchanged
reuse and all required edits. No sample was dropped or retried because it was
slower.

The actual new CI mode was checked against the final diagnostic native binary
and installed Lust. It completed 25 commands with four paired measurements,
fresh runtime and compilation directories, and one admission per native sample.
[`fresh-cache-mode-check.json.gz`](fresh-cache-mode-check.json.gz) remains
unqualified because four pairs are below the required twelve.

## Identities and reproduction

| Diagnostic executable | SHA-256 |
| --- | --- |
| Reference before cache | `600086c4e7b0b22063376205f36d4a648f73423f5ec0cd30cf81dc2c83b59dbf` |
| Candidate before lazy formatting | `94d337fdb9571805276ab8d71f19a2692aa45b78140a6b43cdcdb5d54831310d` |
| Final candidate | `734d50867257bfd1e1105e58ccaf482aa79560007af2069cc22c383cb14720fa` |

The final change delays formatting native arguments until caching is enabled.
The measured source snapshots are distinct and retained in
[`diagnostic-sources.tar.gz`](diagnostic-sources.tar.gz), with their runners,
individual file hashes and experiment manifest. The earlier snapshot was
reconstructed by reversing only that two-line change and checked against every
source hash already recorded in the immutable timing report. Build logs are
[`pre-lazy-diagnostic-build.log.gz`](pre-lazy-diagnostic-build.log.gz) and
[`diagnostic-build.log.gz`](diagnostic-build.log.gz).

[`manifest.json`](manifest.json) binds the original and compressed report
bytes, archive members and compiler identities. The compiler binaries remain
identified by hashes; these diagnostic binaries are not release artifacts.

The rejected in-process batching experiment is retained in
[`rejected-backend-batching.json.gz`](rejected-backend-batching.json.gz), with
generated C inputs in [`batching-inputs.tar.gz`](batching-inputs.tar.gz) and its
probe source in the source archive. It was slower than upstream parallel
helpers on all four measured inputs. No batching implementation entered Lustr.
