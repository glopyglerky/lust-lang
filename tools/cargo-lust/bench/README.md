# Measure the coordinator

`measure.py` compares ordinary Cargo with `cargo-lust --engine rustc` or an
explicit Cranelift build. It uses Python 3.10 or later and its standard library.
It does not download dependencies or toolchains.

The default run measures coordinator overhead. Selecting Cranelift measures that
backend on the same Rust toolchain and source. Neither result establishes a
speedup for arbitrary Rust projects.

## Run it

Build the coordinator once, outside the timed samples:

```sh
cargo build --release --manifest-path tools/cargo-lust/Cargo.toml --locked
python3 tools/cargo-lust/bench/measure.py \
  --coordinator tools/cargo-lust/target/release/cargo-lust \
  --engine rustc --samples 6 --output /tmp/cargo-lust-rustc.json
```

Build the pinned unwinding-enabled Cranelift backend, then use that same
toolchain for the baseline and candidate:

```sh
bash tools/cargo-lust/scripts/build-cranelift.sh
source tools/cargo-lust/target/complement/env.sh
python3 tools/cargo-lust/bench/measure.py \
  --coordinator tools/cargo-lust/target/release/cargo-lust \
  --toolchain "$RUSTUP_TOOLCHAIN" --engine cranelift --samples 6 \
  --output /tmp/cargo-lust-cranelift.json
```

Pin `nightly-YYYY-MM-DD` when recording evidence. `--toolchain` resolves Cargo,
rustc and rustdoc with `rustup which` and gives both routes those exact tools.
It does not install anything. Without it, the executable paths come from
`--cargo`, `--rustc`, `--rustdoc`, the corresponding environment variables, or
`PATH`. Use the same installed toolchain for all three.

The benchmark pins `LUST_CARGO` and `CARGO` to the same executable and records the
coordinator's plan before timing. A build must select the requested engine;
`check` must select the compatibility path. An unexpected selection fails the
measurement. The plan must identify the exact Cargo executable and report the
selected library through `codegen_backend`, or `null` for the compatibility path.

For Cranelift builds, `LUST_CRANELIFT_BACKEND` or `--cranelift-backend` must name
the absolute library path produced by setup. The benchmark records its path and
SHA-256, verifies the plan selects it, and checks that its bytes stay unchanged.
This setting is passed only to the candidate. The baseline does not receive it.
The coordinator selects the backend with Cargo command-line configuration;
it does not export profile backend variables into programs or nested Cargo runs.
The rustup preview without the backend's `unwinding` feature does not qualify.

Add `--build-and-run` to summarize compilation plus executable runtime. Every
successful build executes the fixture even without this flag, because a faster
build with wrong behavior is not a qualifying result. Without the flag, those
executions are outside the reported build timing. `--operation check` measures
checking only and does not claim runtime parity. Cranelift normally does not
accelerate the semantic checking phase.

`--samples 2` is a quick smoke run. Use more samples for decisions. Even sample
counts balance which route runs first. `--jobs` defaults to one. Avoid other
builds on the same machine while measuring. Progress goes to stderr; the JSON
report goes to `--output`, or stdout by default.

## What is measured

The workload is a copy of `../fixtures/full-rust`. Its three path-only packages
exercise a build script, generated source, a proc macro, async, unsafe/raw
pointers, generic associated types, const generics, FFI, and panic unwinding
with a destructor. It is a small qualification fixture, not a representative
large application.

| Scenario | State and edit |
| --- | --- |
| `cold` | Each route starts with a fresh Cargo target directory for every sample. |
| `warm_noop` | One excluded warmup, then unchanged source and retained target directories. The report checks that Cargo marks every artifact fresh. |
| `body_edit` | Alternate the exported `benchmark_value` function between returning 41 and 42. The executable must print the edited value. |
| `signature_edit` | Alternate that function's return type between `u32` and `u64`. Its dependent caller already casts the result to `u64`. |
| `rejected_edit` | Replace the return value with a distinct string on each sample. Both routes must reject with Rust's `E0308` type error. No stale binary is executed. |

Body and signature scenarios start with an excluded warmup. The benchmark checks
Cargo's JSON artifact messages to establish that the edited library and its
dependent application both rebuilt. Each pair compiles the same source snapshot
at the same absolute path. The source hash is checked again after each command.
Only the target directories differ. Pair order alternates between baseline first
and candidate first.

Cold means an empty project build cache. The installed standard library,
compiler binaries, filesystem page cache and processor caches remain available.
This does not simulate a freshly provisioned machine.

## Equivalent configuration

Both routes use the host target, the fixture's unchanged default features,
`--offline --locked`, identical job counts and explicit Cargo dev settings:

| Setting | Value |
| --- | --- |
| Optimization | 0 |
| Debug information | 2 |
| Debug assertions | Enabled |
| Overflow checks | Enabled |
| Panic strategy | Unwind |
| Incremental compilation | Enabled |

The program catches a panic and checks that a destructor ran. The benchmark
does not switch to aborting panics to make Cranelift pass. Unsupported Cranelift
configurations fail qualification and receive no speed ratio.

The process owns a temporary Cargo home, source copy, target directories, and
execution directories. It clears inherited build/profile overrides, compiler
wrappers and jobserver flags from the child environment, then records the names
of cleared variables. Values from the user's environment are not copied into
the report. The original fixture, user's Cargo home, toolchains and project
configuration remain unchanged. The copied lockfile is prepared before timing.

The two compiler routes run sequentially. Each successful executable runs once
in its own empty working directory. The comparison checks zero exit status,
exact stdout and stderr bytes, files written, the edited value, and panic/drop
results. A failed command or parity check ends the benchmark. It is never
retried under another compiler.

## Read the report

The JSON contains:

- Raw elapsed nanoseconds, command arguments, exit status, and captured output
  for every measured sample and excluded warmup.
- Per-pair input hashes, ordering, acceptance results and runtime parity.
- The selected compiler plan, exact custom backend path and hash when used,
  Cargo/rustc versions, the rustc commit and host triple, coordinator executable
  hash, and hashes of installed Cranelift backend files when found.
- Fixture files and hashes, explicit profile settings, hardware information,
  and measurement limits.
- Median, nearest-rank p95, minimum and maximum per scenario and route. With six
  samples, nearest-rank p95 is the maximum.
- `baseline_over_coordinator_median`. A value above 1 means the candidate was
  faster in that scenario; below 1 means it was slower. It is a ratio of
  medians, not a guarantee or a statistical significance test.

Build-plus-run timing includes the short coordinator-to-program handoff and
Cargo artifact parsing. Build-only timing includes process startup and compiler
output capture for both routes. The rejected-edit statistic always measures
rejection latency. A failure preserves collected raw data and leaves the
statistics empty, so failed semantics cannot be reported as a speedup.

This first benchmark does not measure peak memory, wide type changes, macro
edits, the upstream Lust daemon, full test-suite time, cross compilation, or
large workspaces. Those remain separate workload and qualification tasks.

## Measure the real Lust compiler project

`measure-project.py` builds the unchanged root Lust crate through ordinary Cargo
and the configured Cranelift route. This measures a real Rust compiler build,
including its native tcc linkage and dependencies. It is a separate workload
from the small synthetic fixture above.

Prepare the coordinator, tcc, and the pinned backend before timing:

```sh
scripts/build-tcc.sh
bash tools/cargo-lust/scripts/build-cranelift.sh
source tools/cargo-lust/target/complement/env.sh
cargo build --release --manifest-path tools/cargo-lust/Cargo.toml --locked
python3 tools/cargo-lust/bench/measure-project.py \
  --coordinator "$PWD/tools/cargo-lust/target/release/cargo-lust" \
  --project "$PWD" --samples 2 --jobs 1 \
  --output /tmp/cargo-lust-real-project.json
```

The coordinator path must be absolute. The selected nightly and custom backend
must already be installed. The script resolves Cargo, rustc, and rustdoc from
that one installed rustup toolchain. It runs `cargo fetch --locked` before the
timed intervals, then uses `--locked --offline` for every build and test. It
preserves the project's normal dev profile instead of using Lust's optimized
`gate` or `release` profile. Inherited backend selections are cleared from the
baseline; the candidate selects the configured library. Backend overrides in
`RUSTFLAGS` or any Cargo rustflag environment variable are rejected. Other build
settings are shared and relevant inherited settings are recorded. The artifact records
must confirm the root binaries use optimization level 0, debug information 2,
debug assertions, and overflow checks.

The real-project benchmark also requires Cargo commit
`3c747ea3ead23951c1b43a5edfcbdb449135ddeb`, supplied by the pinned
`nightly-2026-11-05` toolchain. It sets
`__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT=1` equally for both routes and
their subprocesses. That Cargo revision changed its build-script artifact
layout, while unchanged Lust expects the previous layout. The private temporary
switch restores the layout needed by Lust's build-script handling. Its exact
value, inherited value, Cargo commit, reason, and scope are recorded. A different
Cargo commit fails before measurement because this switch requires renewed
qualification when the pin changes. The general coordinator and small-fixture
benchmark do not apply this compatibility switch.

Both routes build and test with `-vv --color never`. For each test compilation,
cold build, and excluded warmup, the script parses Cargo's actual rustc commands for `lust` and
`lust_core`. The candidate must select the exact configured backend once; the
baseline must use default LLVM or one explicit `llvm` selector. Missing,
duplicated, or overridden backend evidence fails the run. Compiler wrappers and
rustc response files are refused because their effective arguments cannot be
verified from Cargo's command display. The report retains the raw commands and
parsed tokens, including failed checks. A coordinator plan is recorded but does
not establish which backend compiled the binaries.

Two scenarios are measured. Cold builds use new target directories for both
routes, which are removed after each pair to bound disk use. Warm builds keep
separate targets after an excluded warmup and require Cargo to report the
unchanged binaries as fresh. Their paths and hashes must also match the
executables from that route's verified warmup. The first route alternates each pair. Two samples
are the default for an initial run of this large project; the report marks that
sample count as low. Increase the even sample count for performance decisions.
This does not clear the OS page cache or the installed compiler and dependency
sources.

Every successful build must produce both `lust` and `lust-core` under its owned
target directory. Outside the timed build, the resulting Lust executable runs
the existing supported fixture and must print `42`, then checks the async
fixture and must reject it with `L0405`. Version and capabilities output are
also compared. Each route gets a freshly recreated probe directory at the
same absolute path. The daemon and Lust's disk cache are both disabled so one
compiler cannot reuse the other compiler's answer. Only compiler/probe directory
prefixes and the version's build ID are normalized; the report keeps the raw
stdout, stderr, and status for every command, including failures.

Before any timing samples, both routes run the real root `cargo test --bins`
suite in separate empty target directories. Both planned suites run once, even
if one fails, so the report retains comparative failure details. They use
`--no-fail-fast -- --nocapture` to execute all selected bin suites and preserve
test output. Both commands must succeed, their compiled backends must be
verified, and their test counts must match before timing starts. Qualification
targets are then removed. These runs are outside the build-time medians.

Only these two unit-suite subprocesses receive `RUST_MIN_STACK=268435456`.
Lust's production worker stack is also 256 MiB, as defined by `STACK_SIZE` in
`src/pool.rs`; its default 2 MiB test-thread stacks overflow on accepted deep
expressions. The setting, its reason, and any inherited value are recorded for
both routes. All upstream tests and assertions remain enabled. The timed build
and runtime-probe environments retain their original stack setting. Passing
these suites is evidence about this compiler project, not proof for every
Rust program or Lust workload.

The script hashes tracked working files before and after the operations, as
well as tcc, the custom backend, and the coordinator. It does not edit project
sources. Submodule gitlinks are recorded; untracked application source files
are outside this inventory. Use an otherwise idle checkout. Write the report
outside the source tree or as a new file in its root. An existing tracked file
cannot be a report destination. File reports are checkpointed after completed
operations; `--output -` prints one final JSON report. A failed build, compiler
probe, source-integrity check, or test suite retains observations and receives
no speed ratio. Commands are never retried under another backend.
