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

For an installed nightly with `rustc-codegen-cranelift-preview`, use that same
toolchain for the baseline and candidate:

```sh
python3 tools/cargo-lust/bench/measure.py \
  --coordinator tools/cargo-lust/target/release/cargo-lust \
  --toolchain nightly --engine cranelift --samples 6 \
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
measurement.

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
- The selected compiler plan, Cargo/rustc versions, the rustc commit and host triple, coordinator executable
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
