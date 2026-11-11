# Native entry point versus upstream fixtures

The final diagnostic audit passed **all 2,293 cases** in the bounded matrix
below. Both compiler invocations and both program executions succeeded as
expected for every case; matching nonzero exits and signals count as expected
runtime behavior. Fixture files and measured compiler snapshots remained stable.
This is functional evidence, not speed qualification or complete upstream
coverage. `qualified` and `performance_claim` remain false, and the broader
`upstream-accepted-corpus` requirement remains unproved.

The [final report](tmp-spawn-pool-report.json.gz) records 4,586 compiler requests,
4,586 program executions, and two compiler identity queries. Its elapsed time
was 310.98 seconds with two workers. These command timings include concurrent
functional testing and must not be used for performance comparisons.

## Inputs and comparison

Sources are the unchanged top-level Rust fixtures and their support files at
Lust revision `240e08d91d4aed0299639f6f11fd522912b248f9`. Both compilers received
the same absolute source paths. Rust 2024 and the default host TinyCC backend
were used.

| Suite | Cases | Panic | Overflow checks | Debug assertions |
| --- | ---: | --- | --- | --- |
| `tests/pass` | 2,228 | unwind | on | on |
| `tests/unwind` | 60 | unwind | on | on |
| `tests/release` | 5 | unwind | off | off |

The native lane invokes the actual ELF through its Cargo-compatible arguments,
requesting `link,dep-info`; it checks the executable and source/target dep-info.
The upstream lane invokes the actual `lust` ELF with equivalent semantic flags.
Neither lane uses a compiler wrapper or source rewrite. Each program runs as
`./program` in the same recreated per-case directory with the same declared
environment, including `HOME` and `TMPDIR`. Both lanes use `LUST_NO_CACHE=1` and
`LUST_NO_DAEMON=1`. Comparison preserves exit status and signals and uses only the pinned
donor's path, panic thread ID, standard-library path and backtrace-frame masks.

The diagnostic native ELF SHA256 is
`9447fad2740377c02172d091aa4127ccc99ffa6e44edc46528e568db1561ca61`.
The upstream package manifest binds the actual Lust, Lust-core and installed
support hashes to the source pin. The native build recipe is unproved by this
audit; its raw-build support tree was not independently snapshotted. Version
strings, compiler hashes, source context and native working-tree differences are
retained separately. A compiler's printed build hash is not a Git revision.

## Every attempt is retained

[attempts.json](attempts.json) records archive/report hashes, exact runner hashes,
observed compiler-pair counts and the original results. No case was retried
within an attempt. Each full rerun submitted the entire matrix again.

| Attempt | Compiler pairs attempted | Recorded result |
| --- | ---: | --- |
| [Identity preflight](identity-preflight-report.json.gz) | 0 | The first runner incorrectly required the Git revision in a content-hash version string. It stopped before fixture compilation. |
| [Pilot](pilot-report.json.gz) | 18 | All 18 selected module/include/environment/process/panic/nonzero/release cases passed. |
| [Workspace run](workspace-enospc-report.json.gz) | 1,043 | 1,041 passed; the report retains 1,242 unproved rows and 10 artifact/runner failures after filesystem exhaustion. |
| [Temporary-filesystem thread pool](tmp-thread-pool-report.json.gz) | 2,293 | 2,290 passed; three program launches failed with `ETXTBSY` and remain unproved. |
| [Temporary-filesystem process pool](tmp-spawn-pool-report.json.gz) | 2,293 | All 2,293 passed, with no timeouts, launch errors, missing pairs or runtime mismatches. |

The workspace attempt occupied about 222 MiB including filesystem allocation;
its largest raw log was 23,512 bytes. The underlying cause of its transient
`ENOSPC` is unproved. The original report predates explicit attempted-pair
accounting and separate infrastructure error classification, so its `complete`
field must not be read as all compiler pairs having run.

The threaded pool's three `ETXTBSY` errors affected one native and two upstream
program launches. A subprocess fork can briefly inherit another thread's open
executable-copy writer; that is a plausible explanation, not a traced root
cause. The final runner uses two explicitly spawned worker processes with
independent file descriptor tables. No such launch error occurred in its full
run. Recorded available temporary-filesystem space stayed above 4.10 GB.

The archives retain every non-ELF evidence file: raw logs, normalized outputs,
dep-info, per-case metadata, journals, source manifests, exact runners and native
diffs. Compiler/runtime snapshot directories and any leftover case ELFs are
excluded; their identities remain in the reports. Archive member hashes were
verified against the retained originals. The first preflight runner was
reconstructed from its recorded subsequent edits and verified against its exact
recorded SHA256; its recovery note is included.

## Scope and reproduction

This matrix does not cover pass/unwind fixtures under release semantics,
panic-abort profiles, rejection diagnostics, external C/LLVM/sanitizer or
cross-target variants, other upstream gates, Cargo dependency graphs, retained
frontend edits, or any cold/warm speed floor. See the
[acceptance conditions](../../../native-acceptance.md).

Run the [helper](../../../../../tools/check-native-upstream-corpus.py) from the
repository with the intended compiler package and a new output directory:

```sh
python3 tools/check-native-upstream-corpus.py \
  --native /path/to/package/bin/lustr-native \
  --lust /path/to/package/bin/lust \
  --output /tmp/lustr-native-upstream-corpus \
  --workers 2
```

The [validation record](validation.json) binds the final helper and focused test
source hashes to the ten passing helper tests. Complete audit commands and
environments are in the retained reports and raw archives.
