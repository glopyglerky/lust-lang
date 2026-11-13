// Lust adapter for jiff 0.2.37 (docs/ARCH.md section 3.6), features
// `alloc` and `std` (ts-rust's Unix build: `default-features = false,
// features = ["std"]`).
//
// The surface is the one ts-rust uses (execute/tsc/diagnostics.rs and
// project/logging/logger.rs): `tz::TimeZone::UTC`, `tz::TimeZone::tzif`,
// `TimeZone::to_datetime`, `Timestamp: TryFrom<SystemTime>`, and the
// `civil::DateTime` clock fields, plus `Error` with upstream's Display and
// Debug. Everything else is absent, so a program that uses it is a lust
// error (README).
//
// The TZif parser, the POSIX TZ footer rules, transition lookup and the
// civil time math are upstream's own code: `jcore/` is a copy of
// jiff-core 0.1.1's source (the crate jiff 0.2.37 builds on), with the
// edits listed in `jcore/mod.rs`. The files here are line for line ports
// of jiff's thin layer over it.

// Upstream also builds without `std` (core-only, a smaller API); this
// adapter does not have that API, so such a unit is a lust error.
#[cfg(not(feature = "std"))]
compile_error!("the lust jiff adapter implements jiff with the `std` feature only");

pub use crate::{error::Error, timestamp::Timestamp};

pub mod civil;
pub mod tz;

mod error;
mod jcore;
mod timestamp;
