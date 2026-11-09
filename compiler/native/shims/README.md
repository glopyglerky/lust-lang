# lust-shim: the Rust system shim for ts-rust (W-SHIM)

One rustc-built cdylib, `liblust_shim.so`, exports the coarse system
services that ts-rust gets from Rust crates on Linux. It exists so a
lust dev build can call those services through a C ABI instead of
compiling the crates with lust (ARCH 3.7). The release build keeps
the real crates through Cargo and never sees this code.

## Build and test

```sh
cd shims
cargo build --release                    # system allocator
cargo build --release --features jemalloc  # jemalloc (see below)
sh test.sh                               # both builds, every test
```

`test.sh` builds both variants, links `tests/test_shim.c` with the
pinned tcc CLI against the cdylib (the link path spikes/RESULTS.md
K6-4 proves) and once with the system `cc`, runs the C test against
both builds, and diffs the `ORACLE` value lines against
`src/bin/oracle.rs`, a small Rust program that does the same probe
operations directly with the same crates. Any diff is a failure.

The crate is not part of the root package. An empty `[workspace]`
table in its `Cargo.toml` and its own `Cargo.lock` keep it separate,
so a root `cargo build` neither builds it nor breaks on it. When
`lust port prepare` lands it should build this crate once and key the
artifact by lockfile, features, target, toolchain, and ABI version,
as DESIGN-TS section 4 says.

Dependency versions are exact copies of ts-rust's `Cargo.lock`:
rustix 1.1.5, nix 0.31.3, signal-hook 0.4.4, name-to-handle-at 0.1.0,
flate2 1.1.10, jiff 0.2.37, xxhash-rust 0.8.15, num-bigint 0.4.6,
num-traits 0.2.19, tikv-jemallocator and tikv-jemalloc-ctl 0.7.0.
Feature sets are the union of what `ts_goport` and `goport_util`
enable on Linux.

## Boundary rules

`include/lust_shim.h` is the full contract. In short:

- It is a cdylib, never a staticlib through tcc. K6 proved that a
  Rust staticlib final-linked by tcc aborts on a caught panic and on
  Rust thread spawn. The cdylib linked by tcc or libtcc passes.
- Every export catches its own Rust panics (`catch_unwind`) and
  returns a status. `lust_shim_panic_for_test` proves it: the C test
  gets `-1` and the message `lust shim forced panic 42`, on the main
  thread and on a C pthread.
- Status: `0` ok, positive is an errno, `-1` panic, `-2` bad
  argument, `-3` other error, `-4` unsupported build feature. The
  text comes from `lust_shim_last_error`, which is per thread (Rust
  `thread_local!` inside the cdylib; the C test proves two threads
  keep two texts). No C `_Thread_local` is in the test or the header.
- Only scalars, byte buffers, the C records in the header, raw fd
  numbers, and opaque `u64` handles cross. Variable outputs are
  `LustShimBuf` values, owned by the shim and freed only by
  `lust_shim_buf_free`. Never free them with the C allocator.
- Fds: an fd the shim opens stays in a shim table under its raw
  number, the shape ts-rust `fswatch/unix.rs` already uses, and is
  closed only with `lust_shim_close`. Read, write, poll, fstat,
  fcntl, and winsize also borrow fds the shim does not own (stdio,
  C-created) and never close them.
- The one callback export, `lust_shim_readdir_each`, stops at the
  first non-zero callback status and never calls back after it. The
  test proves the stop count.

## What is exported, and why

83 `lust_shim_*` functions. Each one maps to call sites in ts-rust
(`crates/ts_goport/src`). Export only what ts-rust calls is the rule;
the deliberate gaps are listed after the table.

### rustix and nix (fds, files, processes)

- Pipe, read, write, close, open, openat, poll: `fswatch/unix.rs`,
  `bin/tsgo.rs`, `execute/tsc/stdio.rs`.
- fstat, statat, lstat, statvfs: `fswatch/unix.rs`, `bin/tsgo.rs`,
  `execute/tsc/stdio.rs`, `frontend/vfs/osvfs.rs`.
- statx (type and mtime only): the `mod_times` fast path in
  `frontend/vfs/osvfs.rs`.
- getdents batch `read_dirent` and callback walk `readdir_each` over
  `rustix::fs::RawDir`: `fswatch/unix.rs`, `frontend/vfs/osvfs.rs`.
- fcntl F_GETFL: the stdout non-blocking check in
  `execute/tsc/stdio.rs`.
- accessat: `findExecutable` in `execute/tsc/compile.rs`.
- utimensat: `chtimes` in `frontend/vfs/osvfs.rs`. UTIME_OMIT passes
  through as a raw nsec value.
- mknodat: one ts-rust test makes a fifo with it
  (`frontend/vfs/osvfs.rs` dirent test). Kept so that test can run
  through the shim later; it is not used by shipped code.
- dup2 to stdout and stderr: `bin/tsgo.rs` close of stdio.
- Terminal width `tcgetwinsize`: `execute/tsc/compile.rs`. It is the
  only termios call in ts-rust, so it is the only one here.
- getpid, getppid, kill, test_kill, waitid (EXITED and NOWAIT, the
  one wait form used), set parent-death signal: `bin/tsgo.rs`,
  `cmd/tsgo/isprocessalive_unix.rs`.
- getrlimit and setrlimit for NOFILE, AS, DATA: `gostd/rlimit.rs`
  and `gostd/stack.rs`. Other resources give the invalid status.
- prctl THP disable get and set (nix prctl): `thp_guard.rs`.
- Thread signal-mask unblock (nix `SigSet::thread_unblock`):
  `unblock_go_signals` in `bin/tsgo.rs`. It is the only mask call.

Process spawn is not exported. Ts-rust spawns with `std::process`,
never through rustix or nix, so spawn stays with the caller (the
lust std library). The C test forks with libc and waits through the
shim waitid, which is the real split.

There is no epoll export. Grep finds no epoll or eventfd call in
ts-rust; its event waits are `poll`, two fds at a time. When a
program needs epoll, it joins here with its first call site.

### signal-hook (signals)

- Flag handles wrap the `Arc<AtomicBool>` that `flag::register` and
  `flag::register_conditional_default` take: `bin/tsgo.rs`
  (SIGPIPE ignore, the Go dropped signals, the early default actions
  for SIGINT, SIGTERM, and the thrown signals).
- Iterator handles wrap `signal_hook::iterator::Signals`, which is a
  self-pipe queue: `bin/tsgo.rs` signal forwarding and the Go
  runtime start, `cmd/tsgo/main.rs` `signal.NotifyContext`. Wait and
  pending drain to a caller buffer, `next` blocks for one signal,
  `close` ends a drain on another thread, and `add` is
  `add_signal` (the SIGHUP add in forwarding). A drain stash keeps
  batch signals past the caller cap, because a signal-hook pending
  batch is a snapshot.
- `signal_emulate_default` is `low_level::emulate_default_handler`,
  for the SIGPIPE death in `execute/tsc/stdio.rs` and the die from
  signal path in `bin/tsgo.rs`. `low_level::exit` is not exported:
  it is libc `_exit`, one call on the C side.
- Flag registrations do not return a `SigId`. Ts-rust drops it at
  every call site, so a handler lives for the process life here too.
- `lust_shim_kill` takes host signal numbers but only named ones
  (rustix `Signal::from_named_raw`). Ts-rust kills only named
  signals. Real-time signals can still be registered as flags or
  iterated, as `drop_go_signals` does for 35 to 64.

### inotify, fanotify, name-to-handle-at (fswatch)

All from `fswatch/unix.rs`, used by `fswatch/inotify_linux.rs` and
`fswatch/fanotify_linux.rs`. Event records stay in kernel layout:
the caller reads the watch fd with `lust_shim_read` and parses, as
ts-rust does. Fanotify groups keep their nix `Fanotify` value in the
fd table because `fanotify_mark` needs it.

`open_by_handle_at` is not exported. The name-to-handle-at crate has
it, but grep shows ts-rust names it only in a ported Go doc comment
and never calls it. The kqueue and FSEvents backends
(`fswatch/unix_bsd.rs`, notify) are BSD and macOS only, so they are
out of this Linux slice, as are the Windows crates.

### flate2 (gzip)

Whole-buffer encode and multi-member decode: `pprof.rs` writes a
whole profile with `GzEncoder` at the fast level then finishes it,
and `diagnostics_loc.rs` reads whole embedded locale members with
`MultiGzDecoder::read_to_end`. No streaming state is exported,
because no call site needs partial reads or flush points. The
oracle diff pins the encoded bytes, not only a round trip.

### xxhash-rust (xxh3)

One-shot 64 and 128 plus streaming new, update, digest, and
digest128: `frontend/parser`, `binder/lib_snapshot.rs`,
`project/extendedconfigcache.rs`, `tracing.rs`, and more. A 128-bit
digest crosses as low and high `u64` words. No seeded form is
exported; ts-rust never calls one.

### jiff (time)

Zone handles wrap `jiff::tz::TimeZone`: UTC, `TimeZone::tzif` from
raw file bytes, conversion of a Unix timestamp to civil fields, and
a local zone resolver that follows `init_local` in
`execute/tsc/diagnostics.rs` step for step (TZ env, `/etc/localtime`,
the Go zone source list, UTC fallback). It reads the environment at
call time, so the caller calls it once and caches the handle, as
ts-rust caches `localLoc` in a `OnceLock`.

`lust_shim_jiff_format_clock` prints the two clocks ts-rust needs,
in its formats: kind 0 is `format_status_time` (Go `03:04:05 PM`),
kind 1 is the logger clock with millis in
`project/logging/logger.rs`. All other time formatting stays with
the caller, over the civil fields export. A timestamp outside the
jiff range returns the error status, where ts-rust marks the case
unported. The Windows `TimeZone::try_system` path is out of scope
for this Linux slice.

### num-bigint (three whole-value calls)

`jsnum.rs` makes exactly three BigInt calls, each whole-value:
`parse_bytes` then `to_f64` (integer literal parse), `parse_bytes`
then decimal print (`parse_pseudo_big_int`), and `BigInt::from(i64)`
then `pow` then `to_f64` (`Number::exponentiate`). Each is one
export. `out_has` is 0 where Rust gives `None`, so the caller keeps
its Option logic, including the bad-digit case that ts-rust turns
into NaN. There is no BigInt handle and no general arithmetic,
because ts-rust does none through this crate. The `to_i64` and
`to_u32` calls in `jsnum.rs` are num-traits on `f64`, plain float
cast rules in the lust std, so they need no shim call.

### jemalloc (tikv-jemalloc-ctl and the allocator)

With `--features jemalloc`:

- `tikv_jemallocator::Jemalloc` is the `#[global_allocator]` of the
  cdylib, so every Rust allocation inside it and every
  `lust_shim_alloc` call lands in jemalloc.
- `lust_shim_jemalloc_thread_arena_get` and `_set` read and write
  the real `thread.arena` control, and `_opt_narenas` reads
  `opt.narenas`. These are the controls `program.rs`
  (`jemalloc_thread_arena`, `set_jemalloc_thread_arena`) and
  `jemalloc_layout.rs` use. They are never no-ops in this build.
- `lust_shim_jemalloc_layout` is the mechanical half of
  `jemalloc_layout.rs`: one 4 MiB allocation at 4 MiB alignment per
  arena, freed at once, plus the parked `arena-hold` thread on the
  last arena. When to call it (the env flag and the memory-limit
  check) stays caller policy, as it is in ts-rust.

Without the feature, `lust_shim_jemalloc_enabled` gives 0 and the
four controls return `LUST_SHIM_ERR_UNSUPPORTED`. The C test runs
in both builds and checks both answers, including a real arena move
and a 4 MiB-aligned allocation in the jemalloc build.

#### How the lust runtime selects the allocator

Allocator choice is a build choice, not a run-time switch:

1. `lust port prepare` builds this crate twice at most (plain and
   `jemalloc`) and caches each `.so` under its key, features in the
   key. For ts-rust it picks the jemalloc build, because the release
   default feature of `ts_goport` is `jemalloc`.
2. A lust dev binary links or loads the chosen `liblust_shim.so`.
   Selection is which file lands at the link or load path, the same
   way `libbase.so` already works. No code path picks at run time.
3. When the program records a known `#[global_allocator]` (ARCH
   3.7), the lust runtime routes its `alloc` intrinsic to
   `lust_shim_alloc`, `_alloc_zeroed`, `_realloc`, and `_dealloc`.
   With the jemalloc build in place, every lust allocation is a
   jemalloc allocation, so `thread.arena` moves the checker's
   memory for real and the 4 MiB alignment rule holds. With the
   system allocator recorded, the runtime keeps its own path and
   does not route through the bridge.
4. With no recorded allocator, dev builds keep mimalloc as ARCH 3.7
   says, and the bridge exports stay unused.

The bridge exists in both builds, so the same test checks alignment
and realloc behavior against either allocator.

## Numbers

Linux x86_64, this box, release profile as in `Cargo.toml`. Build
times are cold (target dir removed, dependencies in the cargo cache
but not compiled). Link times are the whole command for the C test,
five runs for tcc and three for cc, medians shown.

| Item | Value |
| --- | --- |
| `liblust_shim.so` default | 1,210,744 bytes (943,544 stripped) |
| `liblust_shim.so` jemalloc | 1,787,064 bytes (1,432,248 stripped) |
| cargo build, default, cold | 7.2 s |
| cargo build, jemalloc, cold | 25.5 s (jemalloc C compile) |
| tcc link of the C test | 19 ms |
| cc link of the C test | 322 ms |

`test.sh` prints its own fresh numbers each run (hot builds there
are near zero, and it says so). Warm rebuild after a shim code edit
is one crate recompile plus the cdylib link, well under a second of
work on this box outside the jemalloc C objects, which stay cached.

## Known limits

- Linux x86_64 only. No macOS result exists, as in the K3/K6 spike.
- fanotify needs CAP_SYS_ADMIN. This box does not give it, so the C
  test accepts EPERM from `fanotify_init` there and runs the full
  mark-and-event path only on a host that grants the capability.
  The error path and the wrong-fd EINVAL path always run.
- Borrowed raw fds use `BorrowedFd::borrow_raw`. The caller must
  keep such an fd open for the call, the usual raw fd rule. A closed
  shim fd number that the C side reopens as its own fd is then a
  borrowed fd, which matches how raw fd APIs behave but means
  `lust_shim_close` is the only proof of ownership (it gives EBADF
  for numbers the shim did not open).
- `read_dirent` gives EINVAL when one record does not fit the
  chosen batch size, the same behavior as ts-rust `read_dirent`.
  Pass a batch of 8192 or more, as the test does.
- Signal flags and iterators are process-global state in
  signal-hook. Two queues on one signal both receive it, and a flag
  handler coexists with a queue, as the registry allows. Forked
  children inherit handler state until exec; the test forks only at
  quiet points.
- The shim owns all Rust panic output in the process. It sets a
  silent panic hook once, because the message reaches the caller
  through the last error text (the K6 probe did the same). A dev
  binary that wants Rust panic prints from its own Rust code, if it
  ever has any, would need this revisited.

