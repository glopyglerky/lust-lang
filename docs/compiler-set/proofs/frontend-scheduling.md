# Frontend scheduling contracts

These arguments concern Lustr's command transformation. They are not a proof of
Rust's type system, the parallel compiler implementation, or general speedup.

## Argument preservation

Let `A` be the original Cargo argument vector, beginning with `rustc`. Let
`N > 0` be the explicitly requested frontend job count, and define

```text
F(N) = ["-Zunstable-options", "--jobs-frontend=" ++ decimal(N)]
T(N,A) = A ++ F(N)             if A already contains the compiler separator
         A ++ ["--"] ++ F(N)   otherwise
```

The domain excludes conflicting explicit frontend options, an absent BlueFe
root, other Cargo commands and other compiler engines. Arguments are OS strings;
valid process arguments contain no NUL byte.

For every original argument index `i < len(A)`, the bytes at index `i` in
`T(N,A)` equal the bytes at index `i` in `A`. Proof: this mode first copies `A`
without filtering or decoding its elements, then appends at most three new
elements. Concatenation preserves every existing element and its position.
The additional separator is necessary only when Cargo has not yet entered its
compiler-argument section. Therefore no original package, feature, profile,
target, configuration or compiler argument is removed or reordered.

The transformation takes linear time in the copied argument bytes, plus
`O(log N)` bytes for decimal formatting. This is a coordination cost, not a
compiler-work reduction.

## Configuration boundary

Assume Cargo implements its documented `rustc` interface, the selected tools are
the recorded executables, and user wrappers obey their recorded behavior.
Cargo passes the additional compiler arguments to the selected final target.
Lustr does not replace `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, either compiler
wrapper, any manifest, or project configuration. Explicit BlueFe selection sets
the same four tool-selection variables as the ordinary BlueFe route.

Under those assumptions, the change requests scheduling for the selected
compiler target while leaving dependency selection to Cargo. `--profile check`
is an explicit caller choice, not a profile rewrite performed by Lustr.
Ordinary `check`, `build`, `run` and `test` requests without this option retain
their existing route. Conflicts hidden in configuration or response files are
left for Cargo/rustc to diagnose; Lustr does not silently delete them.

This does not establish that parallel rustc is free of implementation defects,
that diagnostic ordering is identical, or that a user wrapper cannot change
behavior. Acceptance, rejection, runtime and panic/drop tests provide bounded
evidence for the observed toolchain, separately from the argument theorem.

## Performance bound

For total parallelizable work `W`, dependency span `S`, and `N` available workers,
any execution time satisfies `T_N >= max(W/N, S)`. Proof: `N` workers cannot
perform more than `N*T_N` work, and every dependency chain must finish within
`T_N`. Scheduling, memory contention and coordination add costs; the lower bound
does not promise that a real execution approaches it.

If fraction `p` of baseline time `T` improves by factor `s`, with additional
coordination time `D`, the decomposition gives

```text
T_candidate = T * (1 - p + p/s) + D
speedup     = 1 / (1 - p + p/s + D/T)
```

Consequently a gain requires `p*(1 - 1/s) > D/T`. This follows by dividing by
positive `T` and comparing candidate time with `T`. Values of `p`, `s` and `D`
are workload properties to measure. Increasing the job count alone proves no
speedup and can increase memory or regress small targets.

## Executable evidence

`crates/lustr/tests/frontend.rs` checks parser refusal, pure planning, separator
handling and preservation of every nonzero argument byte. Its opt-in produced
compiler test checks both serial and parallel metadata builds, existing project
and encoded environment flags, both compiler wrappers, build scripts and a proc
macro, native runtime behavior, caught panic and destructor execution, a compiled
unit-test harness and rejection of an invalid borrow with `E0502`.

Public-repository timing reports must additionally identify fixed source/tool
hashes, actual root compiler arguments, equivalent features/profiles and raw
paired samples. Different upstream compiler or Cargo revisions remain
confounders. See [the performance model](../bluefe-performance.md) for measurement
assumptions and scope. Tests and bounds do not replace measured speed evidence.
