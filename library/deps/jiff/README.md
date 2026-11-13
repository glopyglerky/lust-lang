# Adapter: jiff 0.2.37

Lust adapter crate (docs/ARCH.md section 3.6) for the parts ts-rust
uses: the status clock and the logger time stamps
(`execute/tsc/diagnostics.rs`, `project/logging/logger.rs`). Features:
`alloc` and `std` (ts-rust's Unix build turns off the defaults and
asks for `std`). Gate: `scripts/test-adapters.sh jiff`.

## Implemented, same results as upstream

- `tz::TimeZone`: `UTC`, `tzif(name, data)`, `to_datetime(timestamp)`,
  `Clone`, and `Send + Sync` (a `static OnceLock<TimeZone>` works).
- `Timestamp`: `TryFrom<SystemTime>` (with upstream's range checks:
  years -9999 to 9999), `Clone`, `Copy`.
- `civil::DateTime`: `hour`, `minute`, `second`, `subsec_nanosecond`,
  `Clone`, `Copy`.
- `Error`: `Display`, `Debug`, `{:#?}`, `Clone`, `std::error::Error`,
  for every error this surface can give (TZif parse errors, timestamp
  range errors, `SystemTime` conversion errors).

The TZif parser (v1 to v4, with upstream's validation and error
values), the POSIX TZ footer rules, the transition lookup, and the
civil time math are upstream's code: `src/jcore/` is a copy of
jiff-core 0.1.1's source (the crate jiff 0.2.37 builds on), made by
`tools/prep_jcore.py`, with the edits listed at the top of
`src/jcore/mod.rs`. `src/*.rs` port jiff's thin layer line for line.
Checked against cargo builds of jiff 0.2.37 with generated probes
(tf-jiff REPORT): every zone file on the box, corrupted and generated
TZif data, `SystemTime` edges, and ts-rust's own `time.Local` code
under many `TZ` values.

## Rejected (lust error, never a different answer)

- Every feature but `alloc` and `std` (`tz-system`,
  `tzdb-bundle-platform`, `tzdb-zoneinfo`, `serde`, ...): L0508.
  So `TimeZone::system`, `try_system`, and `get` are absent.
- Every other item: `Zoned`, `Span`, `SignedDuration`, `civil::Date`
  and `Time`, `Timestamp::now` and the other `Timestamp` methods,
  formatting and parsing (`fmt`, `strftime`, Display and Debug of
  `Timestamp`, `DateTime`, and `TimeZone`), `tz::Offset`, `tz::db`,
  comparisons and hashing of these types, and layout queries (L0652).
