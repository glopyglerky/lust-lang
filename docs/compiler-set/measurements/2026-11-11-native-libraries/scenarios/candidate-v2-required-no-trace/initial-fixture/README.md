# Independent native library contract

Run `tools/qualify-native-library.py` with an actual `lustr-native`, Cargo, and a
separate Rust compiler used only as the oracle. The gate compiles two dependency
packages with the same crate name and different versions, then compiles their
consumer using Cargo's real `--extern` arguments.

The native archive must contain versioned typed metadata and real ELF object
members. The gate verifies public signatures, compiler/runtime/profile identity,
distinct native symbols, and object checksums. Producer source is then hidden.
Replaying the actual consumer invocation must still compile and run from those
artifacts, including unsafe-call checking, scalar ABI boundaries, zero-sized
unit arguments, function-pointer coercion, and conditional compilation. Private,
wrong-type, wrong-arity, wrong-result and unsafe-from-safe calls must fail.
An imported checked-overflow call must exit like the oracle and name the original
producer source line even while that source is hidden.

Corrupt or absent metadata/objects and a stock Rust archive must fail. A valid
archive rewritten by the same test helper must still work. Restored sources then
exercise real Cargo freshness, private-body edits, a changed signature that
invalidates the consumer, and valid recovery.

Public generic, struct, constant and macro exports, and `must_use`, `deprecated`
and explicit linkage attributes, are valid Rust fixtures but
remain explicit unsupported native contracts. Producing a successful library
while silently omitting one of these exports fails this gate. Public data layout,
references and ownership across crates, trait/generic metadata, transitive
dependencies, procedural macros and metadata-only `cargo check` remain separate
required milestones. This gate cannot establish a full Rust replacement.

Normal qualification records process execution with `strace`; a runtime rustc or
coordinator invocation in the native lane fails. A restricted local environment
may use `--diagnostic-without-trace`, which records correctness observations but
always leaves `qualified=false` and `no_trace_qualification=true`. It is never a
substitute for the GitHub Actions gate. Elapsed times include instrumentation and
are not compilation-speed evidence.
