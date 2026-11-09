// jiff-core 0.1.1's source (crates.io, MIT OR Unlicense), the crate jiff
// 0.2.37 builds on, copied for the lust jiff adapter. Upstream's text
// except for these edits (each is marked `lust:` where it is made):
// - lib.rs is this file, without its crate attributes and `extern crate`
//   lines; `crate::` paths are `crate::jcore::`, `alloc::` paths `std::`.
// - util/mod.rs: `ArrayStr::as_str` uses a checked `from_utf8` in place
//   of `from_utf8_unchecked` (no `unsafe`); it and `SmallStr::as_str` are
//   not `const fn`; `ArrayStr::<N>` / `SmallStrKind::<N>` are written out
//   in three constructors; `impl fmt::Write for ArrayStr` is cut (tests
//   only).
// - bounds.rs: the boundary types and `RawBoundsError` derive Clone and
//   Copy (upstream writes `impl<B> Copy` and `impl<B> Clone`); one
//   `<Self::Primitive as Primitive>::from_i64`; `Primitive`'s impls call
//   `<$intty>::try_from` and the method forms of `checked_add` and
//   `checked_mul`; `impl From<f64> for Sign` is cut (`#[cfg(any())]`).
// - civil/weekday.rs: the two `FusedIterator` impls are cut.
// - civil/date.rs: the `div_ceil` helper of `Date::weekday` is a
//   module-level fn (`weekday_div_ceil`).
// - civil/time.rs: `buf[end - 1]` for `buf[usize::from(end) - 1]`.
// - tz/posix.rs: `loop` for the two `for _ in 0..` loops.
// - tz/tzif/mod.rs: `MaybeNamedTimeZone::name` spells out `as_deref`.
// - tz/tzif/parser.rs: `lust_split_at(x, n)` (std's `split_at`, at the
//   end of that file) for `x.split_at(n)`; `while let Some(chunk) =
//   it.next()` for `for chunk in &mut it`; `(1..=bytes.len()).zip(..)`
//   for `(1..).zip(..)`.
// None of the edits changes a result or a panic. Regenerate with
// library/deps/jiff/tools/prep_jcore.py.

/*!
A small collection of datetime primitives to support Jiff.

The primary motivation of this crate is as an implementation detail for
the [Jiff](https://docs.rs/jiff) crate. Indeed, if you're seeing this
documentation, Jiff is probably the crate you want, not this one.

# Motivation

The primary motivation for this crate's existence is so that the `jiff` and
`jiff-static` crates can share code. Prior to the birth of `jiff-core`,
there were some major hacks involved that permitted code sharing by way of
duplication. Therefore, some chunk of code was compiled twice if you depended
on both `jiff` and `jiff-static`.

A secondary motivation is that `jiff-core` provides a useful set of primitives
for datetime handling that others may find useful if they don't want to depend
on Jiff proper. For example, callers looking to convert between timestamps and
datetimes may do so with this crate:

```
use jiff_core::{civil, tz::Offset, Timestamp};

let ts = Timestamp::UNIX_EPOCH;
assert_eq!(ts.to_datetime(Offset::UTC).date(), civil::date(1970, 1, 1));
```

Callers can perform the reverse operation too:

```
use jiff_core::{civil, tz::Offset, Timestamp};

let datetime = civil::date(1970, 1, 1).at(0, 0, 0, 0);
assert_eq!(datetime.to_timestamp(Offset::UTC).unwrap(), Timestamp::UNIX_EPOCH);
```

The above operation is fallible because, like Jiff proper, not all civil
datetimes can be combined with all offsets to produce an instant within this
crate's valid bounds (which it shares with Jiff):

```
use jiff_core::{civil, tz::Offset, Timestamp};

let datetime = civil::date(9999, 12, 31).at(23, 59, 59, 999_999_999);
assert!(datetime.to_timestamp(Offset::UTC).is_err());
// The maximum datetime can be used to produce a timestamp only by using the
// maximum offset from UTC. If any other offset were permitted here, it would
// imply the ability to get a timestamp corresponding to a civil datetime in
// the year 10,000 CE. (And similar for the minimal datetime.)
assert_eq!(datetime.to_timestamp(Offset::MAX).unwrap(), Timestamp::MAX);
```

# What does this crate not do?

The major missing pieces from this crate are:

* Formatting and parsing, although callers may find the `Debug` trait
  implementations of types in this crate to be useful.
* Convenient time zone aware handling.
* Platform integration with the [Time Zone Database].
* Any kind of duration type.
* Good documentation demonstrating proper usage of the crate.
* Maturity and stability. Users of this crate should expect more breaking
  change releases than Jiff proper.
* There is no way to convert a `jiff-core` type directly into a `jiff` type
  or vice versa. You must go through the proper constructors. These conversions
  are intentionally missing so that `jiff-core` is not a public dependency of
  `jiff`.

[Time Zone Database]: https://www.iana.org/time-zones
*/


pub use self::timestamp::Timestamp;

#[macro_use]
mod logging;
mod macros;

pub mod bounds;
pub mod civil;
pub mod constants;
mod timestamp;
pub mod tz;
pub mod util;
