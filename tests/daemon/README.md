# Daemon edit scenarios

Each directory is one edit sequence: `v1.rs`, `v2.rs`, and sometimes
`v3.rs` (single-file), or `v1/`, `v2/` directories that hold a whole
file tree with `main.rs` as the entry (multi-file).

`scripts/test-daemon-scenarios.py` runs every version in order against
one warm daemon, a cold lust, and rustc. The warm answer must match
both. A scenario where lust rejects valid Rust in both modes passes
with a note: it starts testing for real the day the feature lands.

The set covers the cross-item value and layout flows of ARCH 8.1-8.3:
const and static values from const fns (also transitively), consts in
patterns, array lengths, discriminants, `size_of`/`align_of`/
`offset_of!`, type aliases, cfg-gated items, macro-generated items,
derived impls (Debug/Clone/PartialEq) under field and variant edits,
and the same flows across module files. The `review_*` dirs are the
wave-A1 review repros (B1-B3, C3 in the corelang Opus review). The
`glue*` dirs edit an Adt in a version that does not reach its glue
(drop, Debug, Clone), then reach it again: the glue must not come
back stale (the edit fuzzer's glue gap detours cover the same class).

Cargo mode: a dir with a `cargo` file (`<root> <lust flags>`, for
example `src/bin/c.rs --bin c`) is one Cargo package. Each `vN/` is an
overlay on the package tree; its `.scen` file may `rm <path>`, set
`cmd <root> <flags>` (kept for later versions), or set `env K=V` for
that version's client. The oracle is `cargo run` of the same target.
The `cargo*` dirs edit what Cargo reads besides the sources: bin, lib,
and build.rs discovery, `[features]`, a path dependency's manifest,
edition, `[lints]`, `[profile.dev]`, the `.cargo/config` chain, the
`CARGO_PROFILE_*` env, and workspace members.

The `gs*` dirs add, in one fn body, each name a collect scan looks for
to turn the generic flags on (`generics::flag_scan_name`), on a program
whose kept state has lang generics only: the warm body patch must not
keep the old flags.
