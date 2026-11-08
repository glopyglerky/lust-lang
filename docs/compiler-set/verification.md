# Compiler-set verification

Verification date: 2026-11-08.

## What this establishes

The coordinator preserves ordinary Cargo behavior for the included full Rust workspace, invokes the actual upstream Lust compiler for a supported target, and keeps an unsupported Lust program rejected. The pinned complementary backend with unwind support compiles and executes the full Rust fixture, passes its tests, and compiles and tests the coordinator itself. It also builds the unchanged upstream Lust compiler with a measured cold-development-build reduction, passes the same 391 upstream unit tests, and matches its runtime probes. These results establish the exercised Linux configuration, not an independent full Rust frontend, arbitrary Lust/rustc library interoperability, automatic fast-path selection, or a general Rust speedup.

## Qualified full-project result

The [complete compiler-set workflow at `42ab5dc`](https://github.com/glopyglerky/lustr/actions/runs/42510335761) passed every job. Its [complementary-backend job](https://github.com/glopyglerky/lustr/actions/runs/42510335761/job/128821816027) completed both the small-workspace benchmark and the unchanged-Lust benchmark with the corrected shared command verifier.

The substantial workload builds the original `lust` and `lust-core` binaries from the fork's unchanged upstream compiler source. `src/`, `library/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, and `scripts/build-tcc.sh` have no changes from upstream base `240e08d91d4aed0299639f6f11fd522912b248f9`. Both routes use the same pinned nightly, dependencies, source, default dev profile, and one Cargo build job. The runner reported four available CPUs on an AMD EPYC 9V45, Linux x86_64. Installed toolchains and operating-system caches remain warm; cold means empty Cargo target directories.

| Scenario | Cargo/LLVM median | Coordinator/Cranelift median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 59.536 s | 39.400 s | 1.511 |
| Warm, no source change | 19.532 ms | 21.149 ms | 0.924 |

Cold compilation took **33.8% less time** in two interleaved pairs. The individual LLVM samples were 61.798 and 57.274 seconds; Cranelift took 40.369 and 38.431 seconds. The two no-op pairs showed a small regression. Two samples per route are preliminary evidence, not a statistical significance result or a prediction for other projects. These are build timings; runtime probes, unit suites, backend verification, artifact hashing, and setup are outside that metric.

Before timing, both compiler variants passed 11 client and 380 core unit tests, with the actual compiler commands proving their backend choices. Both cold pairs, the excluded warmup pair, and both no-op pairs then passed supported-program execution, unsupported-program rejection, version, and capability comparisons. Fresh root compiler commands were verified, and no-op binary hashes matched their verified warmups. All 37 source/tool input checks passed.

The benchmark uses two explicit compatibility settings equally for both routes: a 256 MiB minimum stack only for the unit-suite process trees, matching upstream production compiler workers, and the pinned Cargo revision's temporary old-layout opt-out for this legacy Cargo-inspecting project. Timed builds and probes retain their original stack setting. The coordinator does not add either setting to ordinary user builds. The [qualification history below](#first-substantial-project-run) explains why these settings are required.

The [full-project summary and raw samples](evidence/lust-project-42ab5dc-summary.json), [complete compressed report](evidence/lust-project-42ab5dc.json.gz), and [backend provenance](evidence/lust-project-42ab5dc-backend.json) retain the qualified result. The downloaded artifact ZIP matched GitHub's SHA-256 digest before extraction. The report records the actual pull-request merge checkout separately from the reviewed branch head.

### Small workspace on the same runner

The same job also passed 34 fixture pairs, including four excluded warmup pairs, and verified all 68 compiler/artifact records. With six measured samples per route and scenario, it recorded:

| Scenario | Cargo/LLVM median | Coordinator/Cranelift median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 419.15 ms | 325.50 ms | 1.288 |
| Warm, no source change | 12.76 ms | 13.24 ms | 0.964 |
| Function body edit | 107.18 ms | 104.35 ms | 1.027 |
| Function signature edit | 115.86 ms | 111.41 ms | 1.040 |
| Invalid type edit | 36.88 ms | 37.72 ms | 0.978 |

Cold compilation took 22.3% less time on this small fixture. Edit gains remained small, and no-op and rejection cases were slower. Its [summary](evidence/cranelift-linux-42ab5dc-summary.json) and [complete report](evidence/cranelift-linux-42ab5dc.json.gz) preserve this run separately from the historical measurements below. Different runner hardware makes absolute times across separate runs unsuitable for direct comparison.

## Local tools

| Item | Observed identity |
| --- | --- |
| Rust | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| LLVM in rustc | `23.1.1` |
| Host target | `x86_64-unknown-linux-gnu` |
| Host | Linux 6.18.57, glibc 2.39, AMD EPYC 9V74, 9 CPUs available to the process |
| Lust | `lust 0.2.0 (build 2aae8639)` |
| Lust source base | `240e08d91d4aed0299639f6f11fd522912b248f9` |
| Local Cranelift | No installed backend library or nightly toolchain |

The real Lust test used an existing build of the unchanged upstream source at the stated base. The coordinator installs independently and its full Cargo tests require no tcc or registry download.

## Executed tests

```sh
cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline

LUST_TEST_LUST=/path/to/lust \
  cargo test --manifest-path tools/cargo-lust/Cargo.toml --locked --offline \
  --test qualification actual_lust -- --ignored --nocapture
```

The local run used normal Cargo development settings. Baseline and candidate source trees were copies of the same fixture and had separate target directories. Tests did not change project panic settings, features, optimization, debug information, or incremental compilation settings.

The latest local ordinary test suite passes 29 tests. The separately executed actual Lust test also passes. The process-level cases verify unchanged argument bytes, raw I/O, exit status, Unix signal termination, existing compiler wrappers, no execution during planning, and exactly one execution after failure. Regression tests cover explicit rustup selection, grouped release flags, a real Cargo alias that selects release, Cargo configuration placement and precedence, and the requirement for an explicit existing backend file.

The Cargo integration executes a workspace with a separate custom derive proc macro, a generated-source build script, a library dependency, async polling, raw-pointer operations, generic associated types, const generics, a C function calling convention, panic catching, and destruction during unwinding. It checks this output against both ordinary Cargo and the coordinator:

```text
derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1
```

Separate checks exercise workspace tests and a doctest, a manifest path containing spaces, command arguments following `--`, clean Cargo JSON stdout, and rejection of invalid borrowing with `E0502` and the original exit status.

The real Lust path prints `42` for its supported program. A separate async program is accepted by rustc and rejected by Lust with `L0405`. The coordinator does not hide that rejection by switching compilers.

## Optional backend gates

The ordinary test run explicitly ignores the actual Lust and actual Cranelift tests. The Lust test was also executed explicitly as shown above. Cranelift has not been executed locally because the required toolchain is absent.

The compiler-set workflow includes a dedicated, pinned custom Cranelift job. It compiles and runs the full Rust workspace, including `catch_unwind` and a destructor, then executes its tests. A component installation or a constructed command is not a passing backend gate. Inspect the workflow result for the commit in question before describing that backend as qualified.

### Stock backend failure and correction

The first [hosted qualification run](https://github.com/glopyglerky/lustr/actions/runs/42501159516/job/128793996883), on commit `fc16a72e68a6731def4a026aef561ac640f120c3`, installed the stock preview for `rustc 1.101.0-nightly (c980177d8 2026-11-06)`. Its baseline passed. The Cranelift-built program exited 101 instead of completing the panic-recovery fixture. The initial fixture suppressed its panic hook, so the failed execution printed no application output.

Source inspection established that upstream's optional `unwinding` feature supplies the missing catch and cleanup implementation. The integration now requires an explicit custom backend built with that feature, pinned to source `8edf4d040acb77048e04b52a379e93dccae888e6` and its matching `nightly-2026-11-05`. The runtime fixture now prints a deterministic trace before the panic, from its hook, and after recovery. Compilation and direct execution are separate checks. The panic and destructor assertions remain required.

The initial hosted jobs passed Cargo compatibility on Linux and Windows, the Rust 1.85 minimum, and the actual Lust gate. The macOS 14 runner was retired on 2026-11-02, so the CI configuration now uses the supported macOS 15 Arm64 image. Neither the earlier cancelled macOS job nor a failing stock backend counts as a pass.

Cross-platform and minimum-toolchain workflow jobs are distinct from local Linux results. A configured CI job does not count as an executed test.

### Executed custom-backend gate

The [custom-backend job on commit `ceabf7e`](https://github.com/glopyglerky/lustr/actions/runs/42502560395/job/128798273793) passed on Linux x86_64. It built the pinned source with `unwinding`, ran the installation catch/drop probe, compared the full Rust fixture's stdout, stderr and status with ordinary Cargo, and ran the fixture's workspace tests and doctest. It then compiled the coordinator and its test binaries with that same complementary backend and passed all 27 ordinary tests. Actual Lust, ordinary Cargo on Linux and Windows, and the Rust 1.85 minimum also passed in that run.

The corresponding macOS 15 run exposed a test assumption: APFS rejected an invalid UTF-8 executable filename before the coordinator ran. The test now checks arbitrary argument bytes on every Unix host and separately checks invalid UTF-8 executable filenames on Linux. The subsequent [run at `821fcd9`](https://github.com/glopyglerky/lustr/actions/runs/42504515213) passed ordinary Cargo compatibility on Linux, macOS 15 Arm64 and Windows, the Rust 1.85 minimum, and the actual Lust gate. The earlier failed macOS run remains a failure.

The backend used `rustc 1.101.0-nightly (2d20b42b8 2026-11-04)` from `nightly-2026-11-05`, with the normal LLVM-built standard library. Its source, lockfile, build command, binary hash and successful setup probe are retained in [the backend record](evidence/cranelift-linux-backend.json). This qualifies the exercised Linux configuration and cases, not every feature implemented by rustc.

### Explicit mixed-backend qualification

The `821fcd9` run also passed the added mixed-backend test. It kept `fixture-core` on LLVM using Cargo's per-package profile override while compiling `fixture-app` and the procedural macro with the exact configured Cranelift library. The test inspected Cargo's actual verbose rustc commands to verify those selections, then compared the final program's stdout, stderr, panic/drop trace, and exit status with an all-LLVM build.

This establishes the exercised Rust crate boundary with a shared compiler toolchain. It does not exchange native Lust artifacts. Generic code can be instantiated by a dependent crate, so selecting LLVM for a package does not guarantee that all uses of its generic code go through LLVM. The package choice remains explicit and the resulting program still needs qualification.

### Command-scoped configuration regression

The [follow-up run at `98139b0`](https://github.com/glopyglerky/lustr/actions/runs/42507594541/job/128813495199) caught a regression while replacing inherited backend environment variables with command-scoped Cargo configuration. Cargo discarded configuration supplied before the subcommand when the caller also supplied `--config` after it. The mixed-backend test found that the application and procedural macro had no backend flag, although the package override still selected LLVM for the library. The workflow failed before either benchmark ran; the passing output-only checks from that run do not establish the requested compiler selection.

The correction places generated configuration after `build`, `run`, or `test`, before the caller's arguments. Ordinary Cargo tests reproduce the precedence rule with stable optimization settings. Optional execution tests and both benchmark runners inspect actual compiler invocations, rather than relying on a requested plan or a successful program alone.

The [corrected run at `219b915`](https://github.com/glopyglerky/lustr/actions/runs/42508899383/job/128817456020) passed all three actual Cranelift gates, including the mixed-backend override and nested Cargo. It also compiled the coordinator through its own Cranelift route and passed all 29 ordinary tests. Linux, macOS, Windows, Rust 1.85, and actual native Lust checks passed in the same workflow. Doctests execute successfully, but their internal rustdoc compilation is not independently covered by the rustc-command backend assertions.

### Inherited upstream platform failure

The [full upstream Lust macOS job](https://github.com/glopyglerky/lustr/actions/runs/42502560368/job/128798273823) failed to link `lust-core` because `src/image.rs` references Linux's `personality()` function on Arm64 macOS. The file is unchanged from the fork's upstream base (Git blob `44658f0437e5c4c89e442801c8c17b24afe69d5e`). That job used ordinary rustc 1.99.0 and successfully built tcc before the link error. The newer runner exposed an existing source portability gap; a historical macOS 14 result was not observed.

The coordinator's macOS Cargo compatibility job passes independently. Native Lust and the custom complementary-backend bundle remain qualified here on Linux x86_64. The failing upstream job is preserved and is not counted as a pass.

## Performance evidence

The reproducible benchmark in `tools/cargo-lust/bench/` compares equivalent Cargo and coordinated commands in isolated directories, using the same source snapshot and selected toolchain. It retains raw timing and output evidence. The default Rust route measures coordinator overhead. A Cranelift run measures that specific backend and fixture; it cannot establish speed for other workloads.

No general full-Rust speedup is claimed. Upstream Lust's warm-edit ratios describe its qualified subset and workload. They must not be relabeled as measurements of this coordinator.

### Local compatibility-route measurements

The initial release-built coordinator was compared with the same Cargo/rustc 1.99.1 toolchain, one build job, explicit dev settings with unwinding, six interleaved pairs per scenario, and separate target directories. All 34 pairs passed, including four excluded warmup pairs. The 60 measured compilations included 48 successful builds with runtime parity and 12 rejected invalid edits. These records describe the initial fixture and coordinator published in `fc16a72e68a6731def4a026aef561ac640f120c3`; later backend-setup changes are a separate qualification.

| Scenario | Cargo median | Coordinator median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 540.67 ms | 562.78 ms | 0.961 |
| Warm, no source change | 6.84 ms | 8.18 ms | 0.836 |
| Function body edit | 135.55 ms | 135.34 ms | 1.002 |
| Function signature edit | 146.90 ms | 165.23 ms | 0.889 |
| Invalid type edit | 43.34 ms | 42.90 ms | 1.010 |

These are small-fixture observations on a shared host. They show no useful acceleration from delegation alone and include regressions. Six samples do not establish statistical significance or production performance. Nearest-rank p95 is the largest sample at this count. Full configuration, input hashes, samples, and tail values are in [the summary](evidence/cargo-rustc-linux-summary.json); exact command output and raw timing records are in [the compressed JSON report](evidence/cargo-rustc-linux.json.gz).

### Hosted complementary-backend measurements

The passing custom-backend job measured six interleaved pairs per scenario using the same pinned nightly for ordinary Cargo/LLVM and the coordinator/Cranelift route. Both used one build job, optimization level 0, full debug information, overflow checks and debug assertions, incremental compilation, and panic unwinding. The host reported four available CPUs on an AMD EPYC 7763 and Ubuntu 24.04. The benchmark ran after backend installation, so installation time is excluded.

All 34 pairs passed, including four excluded warmups. The 60 measured compilations included 48 successful builds with runtime parity and 12 rejected invalid edits. These records describe commit `ceabf7e82e85a4d9726c2a001b9e237fd8ca04ca` and its fixture; they are distinct from the earlier local compatibility-route run.

| Scenario | Cargo/LLVM median | Coordinator/Cranelift median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 581.36 ms | 455.73 ms | 1.276 |
| Warm, no source change | 18.47 ms | 19.09 ms | 0.967 |
| Function body edit | 146.23 ms | 143.45 ms | 1.019 |
| Function signature edit | 158.57 ms | 155.07 ms | 1.023 |
| Invalid type edit | 52.75 ms | 54.60 ms | 0.966 |

For this fixture, empty-target compilation took 21.6% less time. The edit differences are small; no-op builds and error reporting regressed. Six samples on a small fixture do not establish statistical significance, a useful warm-edit improvement, or performance on production projects. The same toolchain is used within each comparison; timings from the two different hosts and toolchains should not be compared with each other.

The [summary and raw samples](evidence/cranelift-linux-summary.json), [compressed full report](evidence/cranelift-linux.json.gz), and [backend provenance](evidence/cranelift-linux-backend.json) preserve the result beyond the Actions artifact's retention period. The downloaded artifact ZIP's SHA-256 matched GitHub's reported digest before these records were saved.

### Backend-verified follow-up

The `219b915` run repeated the same six-pair fixture workload after the Cargo configuration correction. It checked actual compiler invocations for fresh crates and retained artifact hashes for cached crates, including the build script and procedural macro. All 34 pairs passed: 60 measured observations and eight excluded warmup observations. All 68 backend-verification records passed, covering 140 fresh compiler commands and 108 cached-artifact checks. The same pinned nightly and custom backend source were used within each comparison, on a four-CPU Linux x86_64 runner reporting an AMD EPYC 7763.

| Scenario | Cargo/LLVM median | Coordinator/Cranelift median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 559.07 ms | 434.40 ms | 1.287 |
| Warm, no source change | 17.74 ms | 18.60 ms | 0.954 |
| Function body edit | 139.95 ms | 138.27 ms | 1.012 |
| Function signature edit | 151.85 ms | 146.15 ms | 1.039 |
| Invalid type edit | 50.27 ms | 52.07 ms | 0.965 |

Empty-target compilation took 22.3% less time in this run. Warm edits changed little, while unchanged builds and rejection were slower. These remain small-fixture observations with six samples per route and scenario. No statistical significance, general warm-edit improvement, or production speedup follows from them. Backend verification and artifact hashing are outside the reported build timing; Cargo's verbose command output is included equally in both routes.

The [follow-up summary](evidence/cranelift-linux-219b915-summary.json), [complete compressed report](evidence/cranelift-linux-219b915.json.gz), and [backend identity](evidence/cranelift-linux-219b915-backend.json) retain the new result. Historical measurements above remain separate. The complete workflow still failed its substantial-project verification step, as described below; the successful small-fixture step is not a successful whole-workflow result.

## Substantial-project qualification history

### First substantial-project run

The [initial unchanged-Lust project run](https://github.com/glopyglerky/lustr/actions/runs/42504515213/job/128804138137), at `821fcd9`, built both `lust` and `lust-core` under ordinary LLVM and the custom backend. Both cold pairs, the excluded warmup, and both no-op pairs passed their supported-program, rejected-program, version, and capability checks. The ordinary LLVM unit suite then failed: a build-script test reported failure and the deep-parser test overflowed its libtest thread stack. The report deliberately contains no successful benchmark statistics or speed ratios for that run. The [failure summary](evidence/lust-project-initial-failure-summary.json) and [complete compressed report](evidence/lust-project-initial-failure.json.gz) preserve its raw evidence.

The benchmark's follow-up configuration uses the same 256 MiB virtual thread stack that upstream Lust already gives its production compiler workers (`src/pool.rs::STACK_SIZE`), applied only to both unit-suite process trees. The parser limits and tests are unchanged.

Pinned Cargo `3c747ea3ead23951c1b43a5edfcbdb449135ddeb` also enables a new build-directory layout, while unchanged Lust assumes the old locations and names when reading build-script output. This one project benchmark therefore applies Cargo's exact, temporary `__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1` compatibility setting equally to both routes. It verifies the Cargo commit before using that private setting and records the override. Normal coordinator builds receive neither benchmark compatibility setting. The original test's detailed assertion was lost when the later parser test aborted the process. The subsequent unit suites at `219b915` passed under these explicit shared settings.

See the pinned Cargo sources for [layout](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/compiler/layout.rs), [build-script filenames and output](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/compiler/custom_build.rs), and [the compatibility setting](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/workspace/features.rs).

Review also identified that a planned backend is insufficient proof of execution when Cargo rustflags can override it. The substantial-project runner now verifies the fresh rustc command for each root binary, rejects hidden or ambiguous backend selection, and ties no-op samples to the verified warmup binaries' hashes. Both unit suites run once before any measured builds, preserving failures and withholding ratios unless both succeed.

### Subsequent unit qualification and log-parser correction

At `219b915`, both unit-suite processes returned zero and each reported 11 client tests plus 380 core tests passing. The benchmark nevertheless stopped before timing because its compiler-command verifier rejected a quoted multiline package description in Cargo's verbose output for the `cc` dependency. This was a framing error in the verifier: one Cargo command spanned several physical lines. The raw report contains zero timed samples and no performance statistics.

The [qualification summary](evidence/lust-project-219b915-qualification-summary.json) and [complete compressed report](evidence/lust-project-219b915-qualification.json.gz) preserve both successful test-process results and the failed overall benchmark. The corrected shared verifier preserves complete quoted commands across newlines and continues rejecting malformed or truncated command records. Replaying it against the unchanged saved logs confirms the root test binaries' LLVM and exact custom-backend selections for both 391-test suites. The summary records this retrospective verification separately, including the verifier's source hashes; it does not change the original report's failed status. The later `42ab5dc` run completed the fresh qualification and timed measurements reported at the top of this document.

## Remaining qualification

One small fixture and one substantial compiler project do not prove full compiler conformance. Open requirements include a broader workload census, macro and build-script invalidation, multiple targets and architectures, SIMD/intrinsics, inline assembly, sanitizer and coverage options, no_std/embedded targets, mixed backend ABI correctness, automatic target selection, and performance across representative large projects. Their acceptance conditions remain in the [roadmap](roadmap.md).

## Continuation and bidirectional Rust ABI qualification

The stopped workspace was recovered and its staged files matched the published
commit exactly. [PR #1](https://github.com/glopyglerky/lustr/pull/1) merged the
recovered compiler set and the first continuation changes into `next` at
`62494e27717a64c8612b538409de27048fa6adc2`. The
[recovery record](recovery-2026-11-08.md) identifies all eight matching blobs.

The continuation passed 391 upstream unit tests locally, the 30 coordinator and
entry-point tests, the new image-startup/child-environment regression, the actual
Lust subset gate, and the three existing real Cranelift integration tests. The
native startup change confines Linux's `personality` call and arena allocation
to Linux. Native macOS's full gate remains separate from coordinator portability;
its remaining native support is not established by this Linux run.

A new dependency-free [mixed ABI fixture](../../tools/cargo-lust/fixtures/mixed-abi)
passed the ordinary Cargo path and the real complementary backend on Linux
x86_64 with rustc commit `2d20b42b883dcfc014f3c2c56081eeb6091c2179`, the
`nightly-2026-11-05` pair. The backend was built from the existing pinned source
with unwinding and has SHA-256
`a7ba473e524496e156c6926b92d234feaad8231c6a1136b73e4884aa89e1b04d`.

| Configuration | Shared type crate | Provider crate | Application and callbacks |
| --- | --- | --- | --- |
| Baseline | LLVM | LLVM | LLVM |
| Cranelift consumer | LLVM | LLVM | Cranelift |
| Cranelift provider | LLVM | Cranelift | LLVM |

Every configuration used fresh build output. Actual `cargo -vv` commands
established the backend and common rustc executable for each ordinary crate and
each test harness. The real gate rejects a different compiler commit or host,
missing compilation evidence, wrong backend paths, and conflicting backend
options. Both assignments executed calls in both directions.

The executable and its Cargo test checked Rust-layout aggregates returned by
value, generic instances, const generics, trait objects and their destructors,
nonzero/pointer niches, owned enum variants, and allocation growth or destruction
in the other crate. Panics originating in either crate crossed the other
backend's frames and produced the asserted destructor order. Atomic
compare/exchange, 8,000 concurrent increments, independent thread-local state,
and thread-exit destructors passed. The C ABI callback test used Rust-defined
`extern "C"` functions and a `repr(C)` aggregate; it does not establish
interoperability with a separately compiled C allocator.

The [summary](evidence/mixed-abi-2026-11-08-summary.json) contains the exact tested
source hashes. The [compressed raw report](evidence/mixed-abi-2026-11-08.json.gz)
retains each command's stdout, stderr and exit status as base64-encoded bytes,
compiler identity, and success receipts. Failed earlier benchmark reports remain
unchanged. The new mixed ABI gate measures correctness, not speed, and does not
claim native Lust rlib interoperability or general Rust/backend equivalence.
