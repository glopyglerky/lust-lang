# Adapter: nix 0.31.3

Lust adapter crate (docs/ARCH.md section 3.6) for the two prctl
forms ts-rust's thp_guard.rs uses, the fanotify forms
fswatch/unix.rs uses, and the kqueue forms fswatch/unix_bsd.rs
uses on Darwin. Gate: `scripts/test-adapters.sh nix`.

## Implemented, same results as upstream

- `sys::prctl::set_thp_disable` and `get_thp_disable` over
  prctl (the sysx shim), with nix's `Result` form.
- `sys::fanotify` (feature `fanotify`): `Fanotify::init` and
  `Fanotify::mark` over fanotify_init(2) and fanotify_mark(2),
  the `MaskFlags`, `InitFlags`, `EventFFlags`, and `MarkFlags`
  flag types, `FANOTIFY_METADATA_VERSION`, and the `AsFd`,
  `AsRawFd`, `FromRawFd`, and `From<Fanotify> for OwnedFd`
  impls. Events are read by callers as raw bytes on the fd,
  as ts-rust does.
- `NixPath` for `[u8]`, `str`, `OsStr`, `CStr`, `Path`, and
  `PathBuf` (interior NUL is EINVAL, as upstream).
- `sys::signal` (tf-front2): the `Signal` enum (features
  `process` or `signal`; Linux numbers, `non_exhaustive`, the
  upstream derives) and, with `signal`, `pub use Signal::*`,
  `SigSet` (`empty`, `add`, `remove`, `clear`, `contains`,
  `thread_block`, `thread_unblock`, `Extend` and
  `FromIterator` of `Signal`, `Clone`, `Copy`). The mask calls
  are glibc's `pthread_sigmask`, as upstream's.
- `sys::prctl` needs feature `process` and `sys::fanotify`
  feature `fanotify`, as upstream.
- `errno::Errno` (also `nix::Error`): upstream's Linux
  `#[repr(i32)]` enum with every variant, so `err as i32`
  works; variant-name Debug, `EPERM: Operation not permitted`
  Display from nix's `desc` table, `from_raw` (unknown codes
  are `UnknownErrno`, as upstream), and the `io::Error`
  conversions.
- `sys::event` (feature `event`, Darwin): `Kqueue::new` and
  `Kqueue::kevent` over kqueue(2)/kevent(2) (the sysx shim),
  `KEvent` (new and the six accessors), the `EventFilter`
  enum, and the `EvFlags`/`FilterFlag` flag types, with
  Darwin values as upstream passes libc's. Upstream's module
  is BSD-only; here it exists on macOS only.
- `sys::time::TimeSpec`: `new`, `from_duration`, `tv_sec`,
  `tv_nsec`, the `UTIME_NOW`/`UTIME_OMIT` consts (the
  target's values), `AsRef`/`AsMut` of the adapter's
  `Timespec` record, and the `Duration` conversions.
- `errno::Errno` on macOS is the same enum form with
  Darwin's errno numbers and variant set (from sys/errno.h;
  `desc` texts still nix's).

## Rejected (lust error, never a different answer)

- Everything else in nix, including fanotify `read_events`,
  `FanotifyEvent`, `write_response`, the Errno forms that
  read or write the C errno (`last`, `set`, `clear`,
  `result`), `kevent64`, the `poll` feature's forms (the
  feature name is listed because upstream's `event` pulls it
  in; its items reject), and the rest of `sys::time`.
  `sys::signal`, `sys::prctl`, and `sys::fanotify` are
  absent off Linux (their numbers here are Linux values).
  Layout queries on adapter types reject. Flag and `KEvent`
  Debug forms are tuple/record forms, not upstream's
  bitflags/libc forms.
