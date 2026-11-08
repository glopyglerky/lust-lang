# Compiler-set verification

Verification date: 2026-11-08.

## What this establishes

The coordinator preserves ordinary Cargo behavior for the included full Rust workspace, invokes the actual upstream Lust compiler for a supported target, and keeps an unsupported Lust program rejected. The pinned complementary backend with unwind support also compiles and executes the full Rust fixture, passes its tests, and compiles and tests the coordinator itself. These tests establish an executable integration. They do not establish an independent full Rust frontend, arbitrary Lust/rustc library interoperability, automatic fast-path selection, or a speedup on production Rust projects.

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

The correction places generated configuration after `build`, `run`, or `test`, before the caller's arguments. Ordinary Cargo tests reproduce the precedence rule with stable optimization settings. Optional execution tests and both benchmark runners inspect actual compiler invocations, rather than relying on a requested plan or a successful program alone. The corrected code requires a new hosted qualification.

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

## Remaining qualification

### First substantial-project run

The [initial unchanged-Lust project run](https://github.com/glopyglerky/lustr/actions/runs/42504515213/job/128804138137), at `821fcd9`, built both `lust` and `lust-core` under ordinary LLVM and the custom backend. Both cold pairs, the excluded warmup, and both no-op pairs passed their supported-program, rejected-program, version, and capability checks. The ordinary LLVM unit suite then failed: a build-script test reported failure and the deep-parser test overflowed its libtest thread stack. The report deliberately contains no successful benchmark statistics or speed ratios for that run. The [failure summary](evidence/lust-project-initial-failure-summary.json) and [complete compressed report](evidence/lust-project-initial-failure.json.gz) preserve its raw evidence.

The benchmark's follow-up configuration uses the same 256 MiB virtual thread stack that upstream Lust already gives its production compiler workers (`src/pool.rs::STACK_SIZE`), applied only to both unit-suite process trees. The parser limits and tests are unchanged.

Pinned Cargo `3c747ea3ead23951c1b43a5edfcbdb449135ddeb` also enables a new build-directory layout, while unchanged Lust assumes the old locations and names when reading build-script output. This one project benchmark therefore applies Cargo's exact, temporary `__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1` compatibility setting equally to both routes. It verifies the Cargo commit before using that private setting and records the override. Normal coordinator builds receive neither benchmark compatibility setting. The source-backed layout diagnosis still requires the full follow-up suites to pass; the original test's detailed assertion was lost when the later parser test aborted the process.

See the pinned Cargo sources for [layout](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/compiler/layout.rs), [build-script filenames and output](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/compiler/custom_build.rs), and [the compatibility setting](https://github.com/rust-lang/cargo/blob/3c747ea3ead23951c1b43a5edfcbdb449135ddeb/src/workspace/features.rs).

Review also identified that a planned backend is insufficient proof of execution when Cargo rustflags can override it. The substantial-project runner now verifies the fresh rustc command for each root binary, rejects hidden or ambiguous backend selection, and ties no-op samples to the verified warmup binaries' hashes. Both unit suites run once before any measured builds, preserving failures and withholding ratios unless both succeed.

### Broader coverage

The fixture does not prove full compiler conformance. Open requirements include production dependency graphs, macro and build-script invalidation, multiple targets and architectures, SIMD/intrinsics, inline assembly, sanitizer and coverage options, no_std/embedded targets, mixed backend ABI correctness, automatic target selection, and performance on representative large projects. Their acceptance conditions remain in the [roadmap](roadmap.md).
