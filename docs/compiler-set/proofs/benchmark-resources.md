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

Private delivery evidence records actual tool/source/dependency hashes, commands,
passed/failed/unrun checks, first failures, retained artifacts and cleanup. It is
not embedded in public source because host paths and raw output are private.
See [measurement protocol](../benchmark-method.md#optional-linux-cpu-and-peak-rss)
for scope, cumulative-max counterexample, overhead and inference limitations.
