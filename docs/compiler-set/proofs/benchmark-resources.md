# Optional resource collector qualification

This extension measures Linux waited-child CPU and largest waited-process peak
RSS, not aggregate memory or energy. It does not change compiler selection,
semantic gates, lane order, seeds, metadata projection or statistical estimators.
No public workload measurements or candidate/control comparison are part of this
qualification. Existing historical measurements remain wall-time-only.

The benchmark workspace alone adds pinned nix 0.31.3 with resource/process/signal
features; the independently installable coordinator remains registry-free.
The safe API and units were checked against the actual cached crate source and
[Linux getrusage(2)](https://man7.org/linux/man-pages/man2/getrusage.2.html).

Focused fixtures exercise positive CPU/RSS, large then small memory allocations,
waited grandchildren, byte output and non-UTF8 argument/environment transport,
null payload stdin, expected nonzero exit, signal preservation, fresh-only report
publication and parent-lease cancellation. Protocol validation rejects missing,
malformed, wrong-schema/scope/request/collector/PID, negative counters and
status-inconsistent reports. Real tiny-project gates additionally exercise
runner timeout/output-cap failure with unavailable usage and null summaries.
These synthetic observations qualify machinery; they are not compiler benchmarks.

Protocol v2 separates collector completion from payload outcome. Assume the
pinned collector wrote the bound report, no outside writer replaced it, and the
OS wait status is reliable. A valid report with collector exit zero returns the
payload status s unchanged. Qualification then requires s.code() to equal the
expected numeric exit. For every signalled s, s.code() is absent, so that equality
is false. Thus a successful collector cannot turn a signal into an accepted
numeric failure. Collector failure or an invalid report yields no conclusion.
This is a conditional argument about the protocol, not compiler correctness or
authentication against malicious same-user payloads.

Real regressions exercise SIGPIPE using a Rust payload compiled by the selected
nightly with default broken-pipe termination, and SIGTERM with an inherited
blocked collector mask. An isolated fixture verifies that a non-leader collector
rejects its invocation before starting lease cleanup and leaves its sibling alive.
All affected process groups belong to the fixtures. The group-leader condition
prevents that misuse; deliberately joined groups and detached sessions remain
outside a universal ownership or containment claim.

The corrected protocol passed 13 runner unit tests, one gate unit test and seven
enabled integration tests, including both resource tests. Four statistics replay
checks also passed. Formatting and matching Clippy with warnings denied passed.
Qualification used an unreleased matched stage2 control built from BlueFe
`1b74388e196ee8813f38ba9eeb6d4ab43b6b1071`, Rust 1.102.0-dev, with official
Cargo 1.99.1. Its runtime manifest SHA256 is
`cd7ca233f21093aa3f18be2a6318fe305eeae787b837c00d676079400a6c78e4`.
An outer-runner regression retained SIGPIPE as signal 13 with available usage
and null conclusions, matching the disabled protocol's payload outcome.
These are Linux qualification observations, not new public workload timings.

Private delivery evidence records actual tool/source/dependency hashes, commands,
passed/failed/unrun checks, first failures, retained artifacts and cleanup. It is
not embedded in public source because host paths and raw output are private.
See [measurement protocol](../benchmark-method.md#optional-linux-cpu-and-peak-rss)
for scope, cumulative-max counterexample, overhead and inference limitations.
