# Native Cargo preview qualification — 11 November 2026

Both the standalone Lustr package and the combined BlueFe/Lustr package passed
their bounded Linux preview qualification. The downloaded archives were then
checked against their manifests, build receipts and recorded compiler identities.
`native_full_rust` and `full_native_speed_qualified` remain **false**.

This record preserves the complete package command records and raw timings from
the successful runs. The measured implementation remains at the commits below;
this evidence-only commit does not change either measured package.

## Source and downloads

| Package | Source | Successful run | Package artifact | Evidence artifact |
| --- | --- | --- | --- | --- |
| Standalone Lustr with official Rust 1.99.1 | Lustr `f1c4bf2c6a22baa2ea146bb270804be547c15473` | [42993690480](https://github.com/glopyglerky/lustr/actions/runs/42993690480) | [13843212655](https://github.com/glopyglerky/lustr/actions/runs/42993690480/artifacts/13843212655) | [13843277488](https://github.com/glopyglerky/lustr/actions/runs/42993690480/artifacts/13843277488) |
| Combined BlueFe/Lustr | BlueFe `7175cffbb56b0e596a41f0bbda7ea58d35127502`, Lustr `f1c4bf2c6a22baa2ea146bb270804be547c15473` | [42993768154](https://github.com/glopyglerky/blueFe/actions/runs/42993768154) | [13843567817](https://github.com/glopyglerky/blueFe/actions/runs/42993768154/artifacts/13843567817) | [13843926751](https://github.com/glopyglerky/blueFe/actions/runs/42993768154/artifacts/13843926751) |

The combined package reuses the independently qualified BlueFe component built
from `e091741964e51bc70fc61b1dc14c382809bf8410`. Its Rust implementation matches
upstream `5b7b37aefcb5b2fba3c2392bc9065b9e4602d494`; its installed tools report
Rust 1.102.0-dev and Cargo 1.102.0-dev. The assembly commit changes fork-owned
workflow and package documentation, not that Rust implementation.

Both native builds preserve the complete Lust tree
`0ab1b09815802894e6078de0af59273cbdf46cb0`, pinned to upstream Lust
`240e08d91d4aed0299639f6f11fd522912b248f9`. No upstream divergence was admitted. The [Lust source gate](https://github.com/glopyglerky/lustr/actions/runs/42993690474/job/130432396401) and [BlueFe source gate](https://github.com/glopyglerky/blueFe/actions/runs/42993775569) passed.

GitHub reported expiration on 11 December 2026 for the combined package and
25 November 2026 for its Actions evidence. The standalone artifacts expire on
9 February 2027. These dates describe artifact retention, not support promises.
The records in this directory remain in Git history.

### Inner archive checksums

```text
6576f6248d4eab82dc8e5a5ff473553070ba619df9453def6a217e30911e6c69  bluefe-lustr-native-preview-x86_64-unknown-linux-gnu.tar.gz
df8a0db995a791830be90529dd22e1267329a1e18e54c9d7a6f7498e072ec711  lustr-native-preview-x86_64-unknown-linux-gnu.tar.gz
```

The combined archive is 236,639,663 bytes; the standalone archive is
271,702,064 bytes. Both GitHub ZIP digests and the inner archives' `SHA256SUMS`
matched the downloaded bytes. The audits checked all 345 combined and all 354
standalone manifest payload entries, including hashes, lengths and modes, plus
each manifest itself. No coordinator executable is present.

The combined audit also checked all 145 retained BlueFe component entries and
all 13,138 archived Lustr source blobs against the measured commit. The recorded
`compiler/native/third_party/ts-rust` gitlink remains a benchmark submodule
reference rather than included source content.

## What ran

Each package has the following independent evidence layers:

| Layer | Recorded commands | Timing observations | Result |
| --- | ---: | ---: | --- |
| Package and relocation | 105 | 42 | Passed |
| Native Cargo contract | 207 | 60 | Passed |
| Broader direct native coverage | 38 across 12 fixtures | Single observations only | Required preview gates passed; seven native gaps retained |

The native Cargo compiler ran from the relocated package while original
package/runtime/library locations were unavailable. It compiled the same
dependency-free Rust 2024 binary fixture as the packaged Rust oracle, with
actual runtime output checks. The evidence covers Cargo freshness, source and
included-file changes, environment set/change/unset, feature variants,
incremental-cache isolation, crate identity, panic/overflow settings and invalid
type/move/borrow rejection. Unsupported artifacts and settings produce errors.
Compiler-name sentinels remained unused; native failures did not fall back to
rustc.

The outer qualification's three nonzero exits are expected rejection tests.
The native Cargo qualification's 28 nonzero exits are six unsupported/malformed
requests, eighteen invalid Cargo builds, two checked-overflow/unwind executions
and two expected abort signals. There are no unexpected exits, spawn errors or
timeouts in these layers.

The ordinary-development scenario explicitly clears `CI` and `TF_BUILD` from
its copied environment and leaves `CARGO_INCREMENTAL` unset. Cargo otherwise
disables default incremental compilation when either CI marker is present.
Both lanes retained the profile, runtime, freshness and nonempty cache checks.
Other scenarios preserve their original controlled environments.

The package separately qualifies the explicitly selected complete Rust compiler
with an offline Cargo workspace, library, build script, procedural macro,
panic/drop behavior, tests, doctests and invalid-borrow rejection. Those are
compatibility-compiler results, not native Lust feature support.

## Build receipt

Each archived `lustr-native` hash matches its manifest, build receipt and
measured executable identity. The retained build log contains the actual direct
Rust 1.99.1 bootstrap invocation with optimization level 3, 16 codegen units,
ThinLTO, no debug information and incremental compilation in the owned target
directory. No local opt1 correctness binary was packaged.

| Package | Archived `lustr-native` SHA256 |
| --- | --- |
| Combined | `0660f34d6dfd3800fbbff29124835b861545748ebb939d0e68f3feb07384eada` |
| Standalone | `74dff196e93d47d285ee291776970738a0cac98bcce307f29c8dafb57fcd4cf9` |

CI asserted that packaged `lust` and `lust-core` were byte-identical to the
same-run pristine release build. The downloaded executable hashes also matched
their recorded qualification identities. A separate pristine baseline archive
was not downloaded. Byte equality does not establish exact timing equality.

## Timing results

The tables retain medians from the original raw observations. There were no
benchmark reruns or discarded slow observations during archive inspection.
Do not compare absolute timings across the two different CI hosts/runs.

### Native Cargo

There are six balanced pairs per mode on one small fixture. Values are
milliseconds. The ratio is the median of paired native/oracle ratios, which is
not generally the ratio of the two independently calculated medians.

| Package | Mode | Native median | Rust oracle median | Median paired ratio |
| --- | --- | ---: | ---: | ---: |
| Combined | Fresh target | 110.784 | 166.597 | 0.675562 |
| Combined | No change | 43.061 | 91.187 | 0.470971 |
| Combined | Body edit | 84.153 | 169.936 | 0.482448 |
| Combined | Signature edit | 83.522 | 174.130 | 0.478476 |
| Combined | Cold invalid input | 50.637 | 123.855 | 0.412319 |
| Standalone | Fresh target | 76.380 | 94.443 | 0.821982 |
| Standalone | No change | 21.688 | 43.984 | 0.487941 |
| Standalone | Body edit | 42.933 | 81.065 | 0.527049 |
| Standalone | Signature edit | 42.845 | 77.390 | 0.550766 |
| Standalone | Cold invalid input | 22.643 | 53.397 | 0.427470 |

No-change measures Cargo freshness without invoking the compiler for the crate.
Fresh-target and cold-invalid samples use new Cargo targets and empty native
unit caches; cold-invalid has no preceding valid build. The OS page cache is
not flushed. These are compiler-cache conditions, not a cold-machine claim.

### Original direct Lust

There are seven observations per lane and mode, with actual runtime checks.
Values are milliseconds.

| Package | Mode | Pristine median | Packaged median | Median paired packaged/pristine ratio |
| --- | --- | ---: | ---: | ---: |
| Combined | Process cold | 56.124 | 62.302 | 1.022288 |
| Combined | Daemon no-op | 5.588 | 5.695 | 0.971096 |
| Combined | Daemon body edit | 7.094 | 7.008 | 0.978556 |
| Standalone | Process cold | 37.718 | 39.550 | 1.064895 |
| Standalone | Daemon no-op | 4.030 | 3.772 | 0.925567 |
| Standalone | Daemon body edit | 5.733 | 4.958 | 0.916379 |

Both runs observed a higher packaged cold median and a paired cold ratio above
one. These samples do not establish exact speed preservation or a statistically
proved general regression. Faster native Cargo measurements do not transfer
upstream Lust's speed ratios to arbitrary projects. The native Cargo entry point
uses the unit cache and fresh in-process sessions, not the original daemon or
whole-request response cache.

## Remaining failures and next native contract

The coverage reports retain seven valid-Rust native failures: unsafe pointers,
async/await, generic associated types, dynamic iterators, return-position
`impl Trait` in traits, extern functions and native rlibs. Two positive fixtures
and three invalid-program rejection fixtures pass. This bounded set is not an
exhaustive language inventory.

Native rlib/metadata, external Rust libraries, procedural macro artifacts,
Cargo's test harness, other editions/targets and optimized application profiles
remain unsupported by the new Cargo interface. Its experimental compiler version
is 0.1.0, so higher manifest minimum-Rust-version requirements can reject it.
Only Linux x86_64 with Ubuntu 24.04-compatible libraries was packaged and
qualified. See the [package guide at the measured source](https://github.com/glopyglerky/lustr/blob/f1c4bf2c6a22baa2ea146bb270804be547c15473/docs/compiler-set/native-package.md)
for system libraries and commands.

The [separate pristine-Lust external-input cache regression](https://github.com/glopyglerky/lustr/actions/runs/42993690474/job/130432396509)
still fails. The candidate correction passed its correctness checks but failed
the required faster-results rule; it remains excluded. The
[experiment decision and original timing rows](https://github.com/glopyglerky/lustr/blob/52bbd272741fd591e593e327163e6bfa3d562271/docs/compiler-set/experiments/cache-guard-20261111.md)
are retained separately. The scoped cache-bypass workaround loses warm-cache
performance. The new native Cargo path has its own successful external-input
dependency qualification.

The next concrete native compiler milestone is independently producing a real
library artifact and consuming it through Cargo's `--extern` contract, with
checked metadata, stable cross-crate identities, generics, symbol/linkage and
actual execution. Relaxing CLI rejection or renaming a C archive is insufficient.
The retained native-rlib fixture also requires a rustc consumer; a native-only
artifact would be an intermediate step, not completion of that gate.

## Preserved records

- [Combined package qualification](combined/qualification.json): original complete command records, outputs, source hashes and all 102 timing observations, including the nested native Cargo report.
- [Standalone package qualification](standalone/qualification.json): the corresponding original records from its own run.
- [Combined coverage](combined/coverage.json) and [standalone coverage](standalone/coverage.json): original broader native reports, including failures and stdout/stderr hashes. Their full per-case output files remain in the linked Actions artifacts.
- [Combined build receipt](combined/build-receipt.json), [build log](combined/build.cargo.log), [standalone receipt](standalone/build-receipt.json) and [build log](standalone/build.cargo.log): actual build settings and binary/source/log bindings.
- [Combined audit summary](combined/audit.md), [artifact identities](combined/artifact-verification.json), [evidence audit](combined/evidence-verification.json), [delivery audit](combined/delivery-verification.json) and [standalone audit](standalone/audit.json): independent read-only checks of the downloaded records and bytes.
- [Original BlueFe component qualification](combined/component-source-and-qualification.json) and [payload manifest](combined/component-payload-manifest.json): retained component identities.
- `SHA256SUMS` binds the files in this evidence directory. Audits record observation and scope; hashes alone are not semantic proof.
