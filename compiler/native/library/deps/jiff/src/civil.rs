// jiff::civil: `DateTime` with its clock fields (upstream
// civil/datetime.rs and civil/time.rs). Upstream keeps a `Date` and a
// `Time`; the adapter keeps jcore's `DateTime`, which holds the same two
// values, and reads the fields from its time as upstream does.

use crate::jcore::civil::DateTime as JDateTime;

/// A civil date and clock time, as `TimeZone::to_datetime` returns it.
#[derive(Clone, Copy)]
pub struct DateTime {
    inner: JDateTime,
}

impl DateTime {
    /// Upstream `DateTime::from_jcore`.
    pub(crate) const fn from_jcore(dt: JDateTime) -> DateTime {
        DateTime { inner: dt }
    }

    /// The hour, `0..=23`.
    pub fn hour(self) -> i8 {
        self.inner.time().hour()
    }

    /// The minute, `0..=59`.
    pub fn minute(self) -> i8 {
        self.inner.time().minute()
    }

    /// The second, `0..=59`.
    pub fn second(self) -> i8 {
        self.inner.time().second()
    }

    /// The fractional second in nanoseconds, `0..=999_999_999`.
    pub fn subsec_nanosecond(self) -> i32 {
        self.inner.time().subsec_nanosecond()
    }
}
