# Adapter and layout corpora (Wave A1, layout gate)

This directory is the home of `scripts/test-adapters.sh`. The
Wave A1 part is the layout corpus under `layout/`. Later waves
add std and adapter semantics cases here (ARCH 9.3 runner
table); a missing later subdirectory does not fail this gate,
but `layout/` and `tests/consts/` must exist.

Read `layout/README.md` for the fixture format.

`crates/<package>/` holds the adapter crate cases (ARCH 3.6, W-DEPS):
`deps.toml` pins the upstream version, and every `<case>.rs` is a bin
built by Cargo against the real crate (the oracle, output pinned in
`<case>.stdout`) and by lust with the adapter swapped in. A case headed
`// lust-reject: L####` is valid Rust that lust must reject with that
code. Regenerate a `.stdout` only from the real crate's output.
With `LUST=...`, the runner also prints one `crate <package>:` line of
matched, unsupported, and wrong cases per package (the W-DEPS
exactness table) before the total.
