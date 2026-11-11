# BlueFe integration and performance model

This document describes the transitional rustc-based coordinator. Its proofs
and measurements do not establish the [required native Lustr compiler](native-contract.md).
The [native Lust/Lustr source and its upstream provenance](upstream-lust.md) are preserved;
the current published preview does not contain it.

Lustr coordinates the produced BlueFe compiler with an explicit toolchain root:

```sh
lustr --bluefe-root /absolute/toolchain build --workspace
lustr --bluefe-root /absolute/toolchain test --workspace
lustr --bluefe-root /absolute/toolchain build --target wasm32-unknown-unknown
lustr --bluefe-root /absolute/toolchain doctor
```

The root must contain `bin/cargo`, `bin/rustc`, `bin/rustdoc`, and the compiler's
sysroot libraries. Cargo can be an identified upstream component. A directory
name does not certify compiler provenance. Verify the delivery manifest, compiler
commit and `rustc --print sysroot` before accepting an artifact.

The selector sets `CARGO`, `LUST_CARGO`, `RUSTC` and `RUSTDOC` for this invocation.
It preserves compiler wrappers, project configuration, arguments, profiles and
Rust flags. An inherited explicit `--sysroot` flag can still select another
sysroot. Arbitrary programs that find another compiler through `PATH` retain
that behavior. This is a reproducible invocation recipe, not a sandbox.

`plan` performs no probes. Execution checks the three tools exist before Cargo
starts. `doctor` reports their versions and availability, not backend correctness.
A missing tool is an error; the coordinator never installs or retries a toolchain.
`+toolchain` and the native `--engine lust` route cannot be combined with this
full-toolchain selector.

The separately pinned Cranelift and semantic-inspection compiler versions are
not compatible with BlueFe merely because all are Rust compilers. Build a
complement against the exact BlueFe compiler interfaces and qualify its unwind,
ABI, rejection and runtime behavior before using it. WebAssembly uses LLVM;
native Cranelift qualification does not establish a wasm backend.

## Invocation equivalence

This is a conditional proof about the coordinator's command construction. It is
not a proof of Rust or Lust semantics, operating-system behavior, backend ABI,
or absence of races in user build scripts.

Let `A` be any sequence of OS argument strings after the coordinator options,
`E` the inherited environment, and `R` an absolute complete toolchain root.
Restrict the route to `auto` or `rustc`, excluding the coordinator's `doctor`,
`plan` and help operations. Let `B(R)` map the four variables above to their
corresponding absolute `R/bin` executable paths.

The selected child invocation is:

```text
program     = R/bin/cargo
arguments   = A
environment = E overwritten by B(R)
working dir = the caller's working directory
```

`plan` first selects the Cargo executable from `R`, clones `A` without string
conversion, and constructs exactly `B(R)`. The compatibility branch inserts no
Cargo arguments or removals. `Plan::command` applies those fields directly to
`std::process::Command`; no shell interprets them. On Unix, `exec` replaces the
coordinator with this command. Therefore the command recipe equals the manual
invocation with the same four environment assignments. It also preserves every
argument byte by pointwise cloning. Tests enumerate all nonzero single-byte
values and exercise real process I/O, signals, failures, spaces and non-UTF-8
arguments. Enumeration supports the implementation check; the cloning argument
establishes the general statement under Rust's `OsString`/`Command` contract.

This equivalence does not imply a speedup. In an ideal serial cost model with
unchanged child work `T` and nonnegative dispatch cost `D`, compatibility time is
`T + D`. A faster observed sample needs repeated measurements and an explanation;
forwarding the same command removes no compiler work.

## Bounds on acceleration

Suppose fraction `p` of baseline time can be accelerated by factor `s`, with
coordination cost `D` and baseline time `T`. The model predicts:

```text
candidate time = T * (1 - p + p/s) + D
speed ratio    = 1 / (1 - p + p/s + D/T)
```

A gain requires `p * (1 - 1/s) > D/T`. Even an infinitely fast code generator
cannot improve the unaccelerated frontend, linking and executed-test time.
For parallel work with total work `W`, dependency span `S` and `n` processors,
execution time is at least `max(W/n, S)`. These are algebraic bounds under the
stated decomposition. They do not predict an unmeasured project's `p`, `W`, `S`
or scheduling overhead. See the [rustc codegen guide](https://rustc-dev-guide.rust-lang.org/backend/codegen.html)
and [profiling guide](https://rustc-dev-guide.rust-lang.org/profiling.html).

The produced BlueFe source exposes `--jobs-frontend=N` behind
`-Zunstable-options`. Use it explicitly and give Cargo a bounded job budget:

```sh
CARGO_ENCODED_RUSTFLAGS="$(printf '%s\037%s' -Zunstable-options --jobs-frontend=4)" \
  lustr --bluefe-root /absolute/toolchain check --jobs 4
```

This example supplies the complete Rust flag list. Preserve any existing flags
when adapting it. More threads can increase memory and regress small crates;
no global setting is enabled by Lustr. The Rust team's [parallel frontend report](https://blog.rust-lang.org/2023/11/09/parallel-rustc.html)
provides background, not a transferable speed ratio for this fork.

## Measurements and limits

`crates/lustr/bench/measure.py --bluefe-root ROOT --engine auto` compares
manual Cargo with explicit BlueFe selection on the same compiler. It records
cold, no-op, body edit, signature edit, rejected edit and optional compile-plus-run
metrics with actual per-crate command and artifact verification.

`crates/lustr/bench/study.py` has two separate experiments:

- `subset` builds the upstream real log analyzer with LLVM and explicit native
  Lust, with equal edition, panic, overflow and debug settings. Every build runs
  its result and compares stdout. A stock reference compiler is optional and
  its different revision is recorded. This qualifies one subset program only.
- `frontend` compares one versus four frontend jobs on unchanged full-Rust
  `lust-core` checking. Both routes have the same Cargo job budget and disabled
  incremental compilation. Dependencies are primed in excluded warmups. Each
  timed command rebuilds the root and verifies the actual compiler arguments.
  This measures checking; it excludes root LLVM codegen and linking.

Both alternate pair order, exclude a warmup pair, retain raw samples, record
source/tool hashes and report medians and nearest-rank p95. The paired analysis
uses log timing ratios, 10,000 seeded bootstrap resamples and a two-sided exact
sign-flip test for at most 16 pairs. Intervals assume independent pairs; the
sign-flip null assumes exchangeable paired differences. Sequential shared-host
runs may violate these assumptions. Results are exploratory and use no
multiple-comparison correction. A low p-value proves neither semantic
correctness nor general performance.

The study does not measure peak memory or energy. Hardware and OS cache state,
profile, compiler revision and workload remain part of every result. Keep
rejected or slower candidates in the report. A failed equivalence or source
check yields no speed ratios.

## Work that remains

Full Rust still uses rustc. The existing semantic probe exports inspection data,
not executable MIR or complete cache keys. Native Lust has no compatible rlib
publisher. Automatic target qualification, checked MIR lowering into native
Lust, and a persistent full-Rust frontend remain compiler development projects.

A sound cache requires a complete semantic input identity or an independently
correct invalidation rule. A hash of incomplete inputs cannot establish that
identity. Finite hashes also need an explicit collision assumption; the current
64-bit internal hashes do not constitute a mathematical proof of cache safety.
Future work must track positive and negative reads, trait candidates, macro and
build-script effects, target/profile/toolchain identity, and transitive metadata.
Randomized warm-versus-cold-versus-rustc edit sequences must cover each admitted
reuse class before enabling it automatically.
