# Native performance design

Lustr must meet the cold and warm speed floors in the
[acceptance contract](native-acceptance.md). Cargo owns scheduling; Lustr must
compile both the Lust subset and added Rust features natively. This note records
the upstream lessons and the bounded cache change. It does not establish speed
parity or complete Rust support.

## Build the right reference

At pinned Lust
[240e08d9](https://github.com/t3dotgg/lust-lang/tree/240e08d91d4aed0299639f6f11fd522912b248f9),
the [release profile](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/Cargo.toml)
uses optimization level 3, one codegen unit, ThinLTO and line tables. The gate
profile uses 16 codegen units and incremental compilation for correctness work.
Upstream explicitly excludes it from performance measurement. Its
[build helper](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/scripts/build.sh)
copies gate binaries into `target/release`, so a pathname cannot establish the
recipe. Lustr's build receipt verifies the observed compiler command.

The [release-baseline workflow](https://github.com/glopyglerky/lustr/actions/runs/43069040645)
measured commit
[8110d9f7](https://github.com/glopyglerky/lustr/commit/8110d9f7aad82a8c382ebadcc092248f0db917bc),
which precedes the request-cache change. A cache candidate needs its own release
build and complete qualification. Diagnostic optimization-level-1 results cannot
qualify a delivered compiler. See [CI design](native-ci.md).

The [initial cache release candidate](measurements/2026-11-11-native-cache/cache-candidate/README.md)
was measured separately at commit `5b98ffb6`. Its unchanged warm median was
4.407 ms against Lust's 4.681 ms, with four slower pairs out of twelve; it failed
the strict floor. Disabled-cache cold compilation was 50.840 versus 50.116 ms,
and enabled-cache first admission was 52.408 versus 51.396 ms. Body, signature,
type and macro edits were still slower than Lust. The full-Rust preflights
failed at procedural macro compilation. All samples remain retained and the
archive was withheld. These measurements predate the namespace fix below.

## What Lust's history establishes

| Upstream evidence | Consequence for Lustr |
| --- | --- |
| [Whole-request and unit caches](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/driver/cache.rs) | Reuse already exists. Unit keys include generated C, compiler identity and debug settings; request keys also include source and configuration. A successful request needs complete external-input tracking before reuse. |
| [Frontend images, 7984f608](https://github.com/t3dotgg/lust-lang/commit/7984f608fd1bdd407e65a76c0b4cc139ff66362e) | Images preserve a daemon-style session across process exits through a fixed-address heap. They require allocator, address, stamp and session-lifecycle coordination. Loading and saving are real costs; images are not a free warm path. |
| [Image environment fix, eb71341a](https://github.com/t3dotgg/lust-lang/commit/eb71341adf166d372a4c250a3d3d7d118b1361f3) | Heap snapshots had persisted unrelated environment secrets. A retained design must preserve actual environment dependencies without saving the entire caller environment. |
| [Compiler identity, 04454b57](https://github.com/t3dotgg/lust-lang/commit/04454b577efb587695003b3ce2ac314e2470659a) | A worker must reject another compiler build's request or response, including a replacement at the same installation path. |
| [Lock ownership, 4bdd49a5](https://github.com/t3dotgg/lust-lang/commit/4bdd49a5e9db58c529c2e0a4a1d9b319a2cfbaa5) | Deleting a supposedly stale lock allowed two daemons. Continuous `flock` ownership, rather than PID-file interpretation, fixes that race. |
| [Client environment, 9ea50b6c](https://github.com/t3dotgg/lust-lang/commit/9ea50b6c42e00ff745638bc0be4c7fc003a34f38) | Build-script refresh and execution must use the requesting client's environment. A daemon's startup environment is insufficient. |
| [Daemon bounds, 3697c731](https://github.com/t3dotgg/lust-lang/commit/3697c7313f1ab215709129a515c99bb1f3e47c58) | Retained sessions need eviction, cleanup, a response to hung requests and resource accounting. Retaining every crate indefinitely consumed memory and temporary storage. |

## Implemented reuse

The owned [request cache](../../crates/lustr-native/src/native_request_cache.rs)
reuses upstream whole-request storage for successful, unchanged, dependency-free
binaries. Admission happens after compilation and artifact/dep-info validation.
It checks actual expansion dependencies, source-map entries and resolver paths.
File or environment reads, additional source files, libraries and imported
artifacts exclude admission. Commented or spaced macro syntax cannot bypass
these dependency records.

The namespace binds the native options, encoded request, original environment
digest, build identity, executing compiler inode metadata and caller umask.
Upstream also keys the current TinyCC runtime contents. The cache stores no
copy of the original environment. Lookup verifies source, response and artifact
fingerprints plus executable permissions, stages the result, then publishes it
atomically. The native entry point regenerates Cargo dep-info and artifact
notifications. A failed artifact contract removes the unqualified output.

The cache surrogate uses upstream's synthetic `file:` identity. The original
native request remains bound into its digest and still drives compilation.
This prevents cache-key construction from discovering an enclosing Cargo
manifest and running its build script again. Cargo already supplied the
effective compiler arguments and environment. A sentinel regression covers
initial storage, warm lookup and parent manifest/config edits with unchanged
effective arguments.

Independent review found and corrected three gaps: upstream identity named
`lust-core` rather than the executing `lustr-native`; byte fingerprints omitted
executable permissions; and the key omitted umask. Linux `/proc/self/exe` metadata
also avoids inspecting a replacement binary through its old pathname.

The [functional check](../../tools/check-native-request-cache.py) covers cache
hits, invalidation, corruption, compiler replacement and permissions. The
[retained evidence](measurements/2026-11-11-native-cache/README.md) identifies each
diagnostic binary and its scope. The
[independent review](measurements/2026-11-11-native-cache/independent-review.json.gz)
passed ten checks. It paused TinyCC after frontend consumption, edited the root
to invalid Rust and verified that the completed old-source compile created no
admission. The next request rejected the edited root. A forced dep-info failure
on a warm hit removed the output and emitted no successful artifact notification.

## Rejected batching experiment

An [isolated backend experiment](measurements/2026-11-11-native-cache/rejected-backend-batching.json.gz)
compared upstream parallel TinyCC helpers with serial in-process batches of two,
using identical generated C and 24 interleaved pairs per case.

| Input | Parallel median, ms | Batched median, ms | Slower batched pairs |
| --- | ---: | ---: | ---: |
| Unsafe scalar | 3.60 | 4.64 | 18/24 |
| Unsafe generic | 3.89 | 4.38 | 16/24 |
| 16 functions | 4.31 | 5.13 | 16/24 |
| 256 functions | 7.18 | 8.29 | 18/24 |

Every batched median was slower, so no batching change entered the compiler.
This experiment established no end-to-end cold or warm improvement. The
[paired native diagnostics](measurements/2026-11-11-native-cache/paired-diagnostic.json.gz)
also retain uncached cold regressions and warm outliers. Their cold lane disables
caching. A separate
[first-admission experiment](measurements/2026-11-11-native-cache/fresh-cache-diagnostic.json.gz)
starts every sample with empty caches and caching enabled; all four candidate
medians were slower. Neither experiment satisfies the speed floor. CI measures
both cold definitions separately.

## Remaining work

The [phase diagnostic](measurements/2026-11-11-native-cache/phase-profile/README.md)
places most measured cache overhead in storage, rather than source or dependency
validation. Setup medians were about 0.11 ms; stores were 0.58–1.50 ms. This was
instrumented opt1 code on separate temporary storage, so it does not explain
the release regressions by itself. It gives no reason to weaken the inexpensive
correctness checks. The next optimization must demonstrate a same-run release
gain while preserving their guarantees.

Whole-request reuse cannot retain frontend work across body, signature, type or
macro edits. Those modes still require a semantically complete retained design
and separate measurements.

The pinned [worker pool](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/pool.rs)
sizes itself once from `LUST_THREADS` or available CPUs, without Cargo jobserver
accounting. Multiple Cargo compiler processes can therefore multiply worker
counts. The [daemon](https://github.com/t3dotgg/lust-lang/blob/240e08d91d4aed0299639f6f11fd522912b248f9/src/daemon.rs)
serves requests serially. Any retained design must respect Cargo's existing job
budget and preserve independent crate overlap. Full Rust support and both speed
floors remain required before acceptance.
