# Explicit test routes and tool binding

The benchmark helper supports an explicit coordinator route:

```text
lustr-bench-gate tests-via /absolute/toolchain/bin/lustr \
  --bluefe-root /absolute/toolchain --engine cranelift \
  test -p your-package --locked --offline -vv
```

Set `CARGO`, `RUSTC` and `RUSTDOC` to that root's corresponding `bin` paths.
Use a qualified matching `LUST_CRANELIFT_BACKEND` for Cranelift, and supply it
as a hashed external input to a campaign. Pin the coordinator and helper
executables as tools. The legacy `tests <Cargo arguments>` mode still invokes
the explicitly selected Cargo directly.

## Conditional invocation statement

Let `R` be the absolute root, `P` the absolute coordinator executable, `e` one
of `auto`, `rustc` or `cranelift`, and `A` the complete Cargo argument vector.
The helper requires that `A` starts with `test`, contains no `--no-run`, and
that the inherited selectors equal:

```text
CARGO   = R/bin/cargo
RUSTC   = R/bin/rustc
RUSTDOC = R/bin/rustdoc
```

The equalities use exact OS strings, not basenames, substring tests or a guessed
compiler family. Missing or different selectors fail with validation exit 42
before starting the coordinator. Paths are absolute. No shell performs expansion.

For accepted inputs, the child invocation is exactly `P` with arguments
`[--bluefe-root, R, --engine, e] ++ A`, where `++` means sequence concatenation.
`std::process::Command` receives each OS string directly. The helper inserts
nothing into `A`. Under the qualified coordinator's
[root selection contract](../bluefe-performance.md), root
selection sets the three variables to the same values already checked here.
Therefore this route cannot silently select a different root by overriding a
declared selector. This is a statement about command construction, conditional
on the supplied coordinator's behavior and trusted tool identities. It is not
a sandbox or an attestation of an arbitrary executable named `lustr`.

The coordinator's documented backend rules still apply. Caller profiles,
target choices and later configuration can preserve or override a backend
selection. A requested engine alone does not prove which backend executed.
Inspect actual root compiler commands and backend-load evidence for the
configuration being qualified. No project flags or settings should be removed
to make a gate pass.

## Executed checks and remaining limits

The opt-in native fixture uses the produced preview compiler and its matching
unwind-enabled Cranelift component. It runs the same nonempty tests through
LLVM and Cranelift, checks the actual selected-root `rustc --test` command,
and requires the backend path on the Cranelift invocation. The test body catches
a panic and asserts exactly one Drop. Three mismatched or missing core selectors
and a `--no-run` request must fail validation before child-output capture. These are
bounded tests of the real route, not a compiler-equivalence proof.

The helper retains the child's exit status and length-delimited raw stdout and
stderr in its stderr evidence stream. For comparison, it emits the same sorted
test names/counts as the direct-Cargo gate. An unsuccessful child, absent
nonempty test harness or invalid route returns validation 42; that is not
application acceptance. Raw evidence is separate from normalized comparison
bytes. The [measurement method](../benchmark-method.md) defines those limits.

This route qualifies neither every dependency/backend ABI nor all project
tests or doctests on a target. It does not apply `--frontend-jobs` to `test`:
that coordinator option is restricted to Cargo `rustc`. The published
[historical frontend measurements](../measurements/2026-11-09-shared-v2/README.md)
retain their original direct-gate limitation. No historical observation or
speed ratio is changed by this helper addition.
