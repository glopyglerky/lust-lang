# Native request-cache phase profile

This evidence measures the pre-namespace-fix candidate at commit
`5b98ffb63da0139d778ef4a26de28b1d38fd4a0e`. It is an instrumented **opt1 diagnostic**,
not a release speed qualification. No compiler binaries are stored here.

The campaign recorded **210 compiler invocations and 210 successful runtime
checks**. Four fixtures each ran 12 timed pairs plus preflight under both enabled
fresh caches and `LUST_NO_CACHE=1`, using the instrumented and uninstrumented
binaries. Two further requests exercised the old manifest namespace. All samples,
including preflight and slow observations, remain in the report.

## Observations

For sources without an ancestor Cargo.toml, median request-cache work was
0.795, 1.608, 0.737 and 0.797 ms across generics/traits, unsafe scalar, unsafe generic
and multiple-function fixtures. Setup took about 0.106–0.115 ms, missing-marker
lookup about 0.006 ms, and store 0.575–1.499 ms. Store was the main measured cache
cost. Dependency admission and the source reread each took less than 0.009 ms at
the median. The largest observed cache-work total was 6.147 ms; it was retained.

The old cache namespace invoked a logging Cargo sentinel once during first store
and once during a warm hit. Store took 3.144 ms and warm lookup 2.764 ms in that
probe. The sentinel exited immediately without running a real Cargo build. The
namespace was fixed separately; these timings do not describe the fixed binary.

## Conditions and limits

- Compiler build: `gate`, optimization 1, LTO off, 16 codegen units. The copied
  build dependencies were reused; the instrumented build took 2m 23s.
- Only owned Rust source was instrumented. The protected upstream tree was
  unchanged. Fixed atomic counters record `Instant` durations; one JSON line is
  printed after the compiler phases. Nested phases overlap their enclosing
  totals and must not be added to them.
- Each cold request used new private `/tmp` cache, output and runtime directories.
  Workspace space pressure required this layout. The OS page cache was retained.
  These numbers cannot be subtracted directly from earlier workspace-based cold
  comparisons. Whole-process instrumented/control differences varied in both
  directions.
- Setting `LUST_NO_CACHE=1` also changes the existing unit-cache policy. That
  mode's whole-process difference is not solely the request-cache cost.
- The runner checked compiler status, runtime exit status zero, expected runtime
  stdout and dep-info file existence. Runtime stdout and the combined check result
  are retained in the report. Separate raw runtime stderr, runtime exit records
  and dep-info bytes were not retained. This archive does not recreate them.

## Files and verification

| File | Contents |
| --- | --- |
| `report.json.gz` | Complete original report, including all raw timing observations and phase summaries. |
| `build.log.gz` | Original diagnostic build log. |
| `runner.py` | Exact diagnostic runner; its original workspace paths are preserved. |
| `source-and-fixtures.tar.gz` | All 12 owned Rust source files, instrumentation patch, fixture inputs, and original metadata. |
| `raw-output.tar.gz` | All 420 raw compiler stdout/stderr files, runner output and both retained copies of the Cargo-sentinel log. |
| `manifest.json` | Original/stored hashes, archive-member hashes, compiler identities and completed integrity checks. |

Packaging verified all 210 source references against retained fixture bytes,
all 420 raw compiler streams against their archived bytes, and all 106
instrumentation records against the report's phase data. Every timing summary
was recomputed from the retained observations. Both original compiler binaries
were rehashed against the report; neither is copied into this directory.

Gzip headers use an empty filename and timestamp zero. Tar members are sorted
USTAR entries with timestamp, uid and gid zero. The manifest records both
compressed hashes and original/member hashes. The archives were regenerated in
memory to verify deterministic bytes and then reopened to verify every member.

This evidence supports phase attribution under the recorded conditions. It
proves neither cold/warm speed parity nor full Rust or full upstream coverage.
