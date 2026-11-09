# Adapter: tikv-jemalloc-ctl 0.7.0

Lust adapter crate (docs/ARCH.md section 3.6). Gate:
`scripts/test-adapters.sh tikv-jemalloc-ctl`.

This adapter keeps only the items whose output matches the
real crate in every program lust accepts. The w-cinterop3
audit probed the real crate on boat (fresh processes, with
and without `_RJEM_MALLOC_CONF`) and read upstream 0.7.0
source item by item. Anything that cannot match is absent,
so a program that uses it rejects at compile time.

## Implemented, same results as upstream

- `Name`, `AsName` for `[u8]` and `str`, `Name::value_type_str`,
  `Name::as_bytes`, and Debug and Display of `Name`: pure
  functions of the key bytes. The empty and non-null-terminated
  panic messages and the `value_type_str` key list are
  upstream's, copied from its source. Upstream's `as_bytes`
  returns `&'static [u8]`; this one ties the borrow to `&self`,
  so a program that needs the `'static` form rejects in lust
  (a false reject, never a different value).
- `Error` and `Result`: Debug and Display texts are upstream's.
  No kept item can construct an `Error` value (`epoch` and
  `version` reads do not fail in either build), so there is no
  error output to match. Upstream derives `Copy`, `Clone`,
  and `PartialEq` (no `Eq`); so does this adapter.
- `epoch` (a unit struct, as upstream, with associated `read`,
  `write`, `update`, `advance`): a fresh real
  process reads 1 first, each advance adds exactly one, and
  `update` returns the new value. That held across repeated
  runs and with `_RJEM_MALLOC_CONF` set. Nothing else in this
  adapter can move the counter, and the by-name forms that
  could move the real one are absent.
- `version::read`: the exact string
  `5.3.1-0-g81034ce1f1373e37dc865038e1bc8eeecf559ce8\0` of
  tikv-jemalloc-sys 0.7.1, the sys crate the pinned ctl 0.7.0
  resolves to. The driver checks this adapter's version
  against the resolved package, so the string cannot drift.
- The `Access` trait declaration, with no impls for `Name`
  (see below). Features `default`, `use_std`, and
  `disable_initial_exec_tls`.

## Removed in w-cinterop3: values that cannot match

The old adapter modeled one arena: `opt.narenas` read 1 and
every `thread.arena` read 0. The real crate on a 16-core box
reads `opt.narenas` 64 (4 per CPU) and a spawned thread's
`thread.arena` 1; with `narenas:2` in the conf it reads 2.
`thread.tcache.enabled` reads false under `tcache:false`;
the adapter read true. Those values depend on the machine,
the build, and `_RJEM_MALLOC_CONF`, so no adapter over a
different allocator can return them exactly. Removed:

- Every `Access<T> for Name` impl (`u32`, `u64`, `usize`,
  `isize`, `bool`, `&'static [u8]`, `&'static str`). The
  by-name form cannot reject one key at compile time and keep
  another, so all by-name key reads and writes reject,
  including `epoch` and `version` by name. Use the typed
  `epoch` and `version` unit structs above.
- With them go `thread.arena`, `opt.narenas`,
  `thread.tcache.enabled`, the string keys
  (`config.malloc_conf` reads the build's conf string), and
  every `stats.*` and `arenas.*` count.

ts-rust impact: `program.rs` reads `thread.arena` by name and
writes the value back on its checker threads, and
`jemalloc_layout.rs` walks `0..opt.narenas`. Their own output
does not depend on the values, but the same API in another
program prints them, so the API cannot stay. Those files
reject in lust now, and the `ext.tikv_jemalloc_ctl` port
feature tracks that form.

## Rejected (lust error, never a different answer)

- The typed key modules other than `epoch` and `version`
  (`opt`, `thread`, `stats`, `arenas`, `config`, `raw`,
  `stats_print`, `profiling`), `Mib` and `MibStr`, the `mib`
  forms of `epoch` and `version`, and layout queries on
  `Error` and `Name`.
- The `stats` and `profiling` features reject at resolve
  (L0508).
