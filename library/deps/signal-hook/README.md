# Adapter: signal-hook 0.4.4

Lust adapter crate (docs/ARCH.md section 3.6) over the sysx
signal registry. Gate: `scripts/test-adapters.sh signal-hook`.

## Implemented, same results as upstream

- `consts` (and `consts::signal`): the signal numbers upstream
  re-exports from libc on Linux, `FORBIDDEN`, and
  `TERM_SIGNALS`. Registering a forbidden signal panics with
  upstream's message.
- `flag::register`, `register_usize`,
  `register_conditional_shutdown`, and
  `register_conditional_default`, returning a `SigId`.
- `iterator::Signals` (feature `iterator`) with `new`,
  `add_signal`, `pending`, `wait`, `forever`, `handle`, and
  `is_closed`; `Pending<E>` and `Forever<'a, E>` over the sealed
  `iterator::exfiltrator::Exfiltrator` with `SignalOnly` (the one
  exfiltrator here; the Iterator impls are for it alone);
  `Handle` with `close` and `is_closed`. Auto traits as
  upstream (Send and Sync, not UnwindSafe). Pending batches are sorted by
  signal number with duplicates removed, as upstream's slot
  exfiltrator yields them.
- `low_level::raise`, `exit` (a bare `_exit`), `abort`,
  `signal_name`, and `emulate_default_handler` over upstream's
  default-action table (SIGIO and SIGURG and SIGCHLD and
  SIGCONT and SIGWINCH ignore; the stop signals raise SIGSTOP;
  an unknown number is EINVAL).

## Known differences

- Registration replaces any handler installed before it
  instead of chaining to it (signal-hook-registry chains).
  Nothing in a lust program can install a handler today, so
  this shows only against a C library that did.
- At most 24 actions per signal (the C registry's fixed
  table); a 25th registration fails with ENOMEM.
- `Signals` delivery writes one byte per arrival to its pipe;
  a burst that fills the pipe drops later arrivals, where
  upstream's slots keep one pending mark per signal number.
  A program that counts raw deliveries in a burst can differ.
- `SigId` values are an adapter-local counter; there is no
  unregister-by-id form.

## Rejected (lust error, never a different answer)

- `SignalsInfo<E>`, `WithRawSiginfo`, `WithOrigin`, generic
  code over `E: Exfiltrator`, the sealed trait's storage
  hooks, `low_level::channel` and `pipe` helpers, `SigId`'s
  Debug, Hash, and Ord, and layout queries on the adapter
  types.
