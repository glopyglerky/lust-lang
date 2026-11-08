# Compiler-set verification

Verification date: 2026-11-08.

## What this establishes

The coordinator can preserve ordinary Cargo behavior for the included full Rust workspace, invoke the actual upstream Lust compiler for a supported target, and keep an unsupported Lust program rejected. These tests establish an initial executable integration. They do not establish an independent full Rust frontend, arbitrary Lust/rustc library interoperability, automatic fast-path selection, or a speedup on production Rust projects.

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

The ordinary test suite passes 25 tests. The separately executed actual Lust test also passes. The process-level cases verify unchanged argument bytes, raw I/O, exit status, Unix signal termination, existing compiler wrappers, no execution during planning, and exactly one execution after failure. Regression tests cover explicit rustup selection, grouped release flags, and a real Cargo alias that selects release.

The Cargo integration executes a workspace with a separate custom derive proc macro, a generated-source build script, a library dependency, async polling, raw-pointer operations, generic associated types, const generics, a C function calling convention, panic catching, and destruction during unwinding. It checks this output against both ordinary Cargo and the coordinator:

```text
derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1
```

Separate checks exercise workspace tests and a doctest, a manifest path containing spaces, command arguments following `--`, clean Cargo JSON stdout, and rejection of invalid borrowing with `E0502` and the original exit status.

The real Lust path prints `42` for its supported program. A separate async program is accepted by rustc and rejected by Lust with `L0405`. The coordinator does not hide that rejection by switching compilers.

## Optional backend gates

The ordinary test run explicitly ignores the actual Lust and actual Cranelift tests. The Lust test was also executed explicitly as shown above. Cranelift has not been executed locally because the required toolchain is absent.

The compiler-set workflow includes a dedicated, pinned nightly Cranelift job. It must compile and run the full Rust workspace, including `catch_unwind` and a destructor, then execute its tests. A component installation or a constructed command is not a passing backend gate. Inspect the workflow result for the commit in question before describing that backend as qualified.

Cross-platform and minimum-toolchain workflow jobs are distinct from local Linux results. A configured CI job does not count as an executed test.

## Performance evidence

The reproducible benchmark in `tools/cargo-lust/bench/` compares equivalent Cargo and coordinated commands in isolated directories, using the same source snapshot and selected toolchain. It retains raw timing and output evidence. The default Rust route measures coordinator overhead. A Cranelift run measures that specific backend and fixture; it cannot establish speed for other workloads.

No general full-Rust speedup is claimed. Upstream Lust's warm-edit ratios describe its qualified subset and workload. They must not be relabeled as measurements of this coordinator.

### Local compatibility-route measurements

The release-built coordinator was compared with the same Cargo/rustc 1.99.1 toolchain, one build job, explicit dev settings with unwinding, six interleaved pairs per scenario, and separate target directories. All 34 pairs passed, including four excluded warmup pairs. The 60 measured compilations included 48 successful builds with runtime parity and 12 rejected invalid edits.

| Scenario | Cargo median | Coordinator median | Cargo / coordinator |
| --- | ---: | ---: | ---: |
| Empty target directory | 540.67 ms | 562.78 ms | 0.961 |
| Warm, no source change | 6.84 ms | 8.18 ms | 0.836 |
| Function body edit | 135.55 ms | 135.34 ms | 1.002 |
| Function signature edit | 146.90 ms | 165.23 ms | 0.889 |
| Invalid type edit | 43.34 ms | 42.90 ms | 1.010 |

These are small-fixture observations on a shared host. They show no useful acceleration from delegation alone and include regressions. Six samples do not establish statistical significance or production performance. Nearest-rank p95 is the largest sample at this count. Full configuration, input hashes, samples, and tail values are in [the summary](evidence/cargo-rustc-linux-summary.json); exact command output and raw timing records are in [the compressed JSON report](evidence/cargo-rustc-linux.json.gz).

## Remaining qualification

The fixture does not prove full compiler conformance. Open requirements include production dependency graphs, macro and build-script invalidation, multiple targets and architectures, SIMD/intrinsics, inline assembly, sanitizer and coverage options, no_std/embedded targets, mixed backend ABI correctness, automatic target selection, and performance on representative large projects. Their acceptance conditions remain in the [roadmap](roadmap.md).
