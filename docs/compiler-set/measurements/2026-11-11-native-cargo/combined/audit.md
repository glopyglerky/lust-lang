# Final combined BlueFe/Lustr package audit

## Result

The [corrected assembly run](https://github.com/glopyglerky/blueFe/actions/runs/42993768154) passed. Independent inspection of its downloaded archive and recorded qualification evidence found no payload, source, build-receipt or command-result mismatch. No installed compiler was executed and no timing run was repeated during this audit.

This is a qualified preview with an experimental native Cargo path for supported binary crates. **It is not a complete native rustc replacement.** Full Rust compatibility comes from the explicit BlueFe rustc component. The archive has no `lustr`, `cargo-lustr` or `cargo-lust` coordinator executable.

## Download and identity

| Item | Value |
| --- | --- |
| Package artifact | [13843567817](https://github.com/glopyglerky/blueFe/actions/runs/42993768154/artifacts/13843567817) |
| Evidence artifact | [13843926751](https://github.com/glopyglerky/blueFe/actions/runs/42993768154/artifacts/13843926751) |
| Install archive | `bluefe-lustr-native-preview-x86_64-unknown-linux-gnu.tar.gz` |
| Tar archive bytes | **236,639,663** |
| Tar archive SHA-256 | `6576f6248d4eab82dc8e5a5ff473553070ba619df9453def6a217e30911e6c69` |
| Package ZIP SHA-256 | `178d80279cfc40ecc003e45237c8734d507d83335f29d79e9ec275278f67017d` |
| Evidence ZIP SHA-256 | `1edad5955398341b237e973c509766c6c46b513d14356c0c080f28b01fb69b40` |
| BlueFe assembly source | `7175cffbb56b0e596a41f0bbda7ea58d35127502` |
| Lustr source | `f1c4bf2c6a22baa2ea146bb270804be547c15473` |
| BlueFe compatibility component source | `e091741964e51bc70fc61b1dc14c382809bf8410` |

Both ZIP hashes match GitHub's artifact metadata. The archive hash matches the accompanying `SHA256SUMS`. The install directory is `bluefe-lustr-native-preview-x86_64-unknown-linux-gnu`.

The qualified host is Linux x86_64 on Ubuntu 24.04. The recorded requirements are glibc 2.39 or newer; `libstdc++.so.6` with `GLIBCXX_3.4.32`; `libgcc_s.so.1`; OpenSSL 3; libcurl; zlib; and C development headers and a linker. Cargo HTTPS access also needs CA certificates. Other hosts were not qualified by this run.

## Payload and source checks

All **345 listed payload files**, plus `lustr-toolchain.json`, were inspected. Every listed file's bytes, size and permission mode matches the manifest. The archive contains the exact expected file set, no symlinks and no coordinator executable. The manifest itself matches the qualification report's hash and has mode `0644`.

All **145 original BlueFe component entries** retain their exact bytes, sizes and modes. The original component ZIP was independently checked against its GitHub digest. All ten protected Rust source object IDs match in the component commit, assembly commit and pinned upstream Rust tree.

All **13,138 archived Lustr source blobs** match the recorded Git commit, including executable status. The source archive's commit header matches that commit. The native tree remains `0ab1b09815802894e6078de0af59273cbdf46cb0`, the unchanged pinned Lust tree. The optional `compiler/native/third_party/ts-rust` benchmark submodule is represented by its directory marker and recorded Git link; its separate contents are not in the Git source archive.

The package qualification checked that delivered `lust` and `lust-core` are byte-identical to the pristine release build made in this run. Delivered hashes also match the independent native coverage evidence:

| Executable | SHA-256 |
| --- | --- |
| `lust` | `bc76bac00a9664ea03517fa57f66ce6ab96dc22b974dccbc6231fd857e21080e` |
| `lust-core` | `32e524f57704467fd3e69e018e1fe01a6128c634729a50695050739893e442e6` |
| `lustr-native` | `0660f34d6dfd3800fbbff29124835b861545748ebb939d0e68f3feb07384eada` |

The new native executable's hash agrees across the actual archive, manifest, build receipt and measured-tool record. Its receipt binds the source commit, source tree, native tree, helper hash and build log. The log contains exactly one matching direct compiler invocation and one fresh linked Cargo artifact. Actual settings are optimization level 3, 16 codegen units, ThinLTO, compiler debug information off and compiler-build incremental caching on. No optimization-level-one development binary was packaged.

## Qualification evidence

| Evidence set | Recorded commands | Timed compiler/Cargo observations |
| --- | ---: | ---: |
| Outer package and relocation | **105** | **42**, seven pairs in each of three modes |
| Native Cargo | **207** | **60**, six pairs in each of five modes |
| Direct native coverage | **38**, including two identity commands | No repeated performance study |

There were **zero timeouts and zero unexpected command exits**. The outer report's three nonzero commands are required unsupported-input, type-error and borrow-error rejections. The native Cargo report's 28 nonzero commands are specified unsupported-argument, invalid-source and checked-overflow/panic cases. Raw coverage output hashes all match their command records.

Native Cargo checks cover real output, crate/module/file identity, unchanged builds, module and included-file edits, environment set/change/unset, feature variants and explicit incremental selection. Ordinary development also passed with Cargo's actual debug-information-level-two profile and a nonempty incremental directory. That separate scenario explicitly removes inherited `CI` and `TF_BUILD` markers and leaves `CARGO_INCREMENTAL` unset; both lanes follow the same policy. All other scenarios retain their recorded environment policy.

Relocation tests ran while the original package and native runtime/library paths were unavailable. Compiler-name sentinels remained unused. The compatibility path passed the full-Rust fixture's workspace, build script, procedural macro, async/generic constructs, panic recovery, destructor count, unit tests, doctests and borrow-error rejection.

## Recorded medians

These are medians from the retained raw rows, in **milliseconds**. Every successful timed build's executable was checked. The sample counts are small, the OS page cache was not flushed, and the run makes no universal speed claim.

### Pristine Lust and packaged direct Lust

| Mode | Pristine Lust, ms | Packaged Lust, ms |
| --- | ---: | ---: |
| process-cold | 56.123537 | 62.301974 |
| daemon-noop | 5.588346 | 5.694845 |
| daemon-body-edit | 7.093758 | 7.007686 |

The packaged process-cold median was higher in this run. Its median paired ratio was **1.02228754755781**, an observed 2.23% increase across paired samples. The ratio of the two separate medians is a different statistic. These results do not establish exact speed equality or absence of regressions, even though the compiler executable bytes were preserved.

### Native Cargo and BlueFe rustc

| Mode | Native Cargo, ms | BlueFe rustc through Cargo, ms |
| --- | ---: | ---: |
| body-edit | 84.152754 | 169.936227 |
| cold-invalid | 50.636849 | 123.855408 |
| fresh-target | 110.784060 | 166.596820 |
| no-change | 43.060802 | 91.186661 |
| signature-edit | 83.522428 | 174.130107 |

These timings use one dependency-free fixture with its declared development profile, debug information off and Cargo incremental off. The ordinary default development profile is qualified separately. Fresh-target and cold-invalid observations start with new Cargo targets and empty native unit caches. Cold-invalid measures rejection without a preceding valid build. No-change measures Cargo freshness, not a native recompile. Source hashes match within every pair.

## Remaining limits

The direct-native suite has **two positive passes, three correct invalid-program rejections and seven native coverage gaps**. The gaps are unsafe-pointer, async-await, generic-associated-type, dyn-iterator, return-impl-trait-in-trait, extern-function and native-rlib. The experimental Cargo compiler also explicitly excludes native Rust library metadata/rlibs, procedural macros, the Rust test harness, optimized application profiles and foreign targets.

The original direct `lust` path still has the documented upstream external-input request-cache defect. No unadmitted cache patch is included. The separate cache experiment did not meet its faster-than-baseline admission rule. The new Cargo entry point does not reuse that whole-request cache or the upstream daemon; no daemon-latency equivalence is claimed for it.

The earlier [failed assembly run](https://github.com/glopyglerky/blueFe/actions/runs/42991420639) remains preserved. Its default-development incremental assertion incorrectly inherited CI mode. The corrected run keeps the profile and cache assertions and records the narrower environment policy. It supersedes the failed run; the failed run supplied no timing result or downloadable package.

Detailed checks are retained in `artifact-verification.json`, `delivery-verification.json` and `evidence-verification.json`, with the complete downloaded qualification reports and logs.
