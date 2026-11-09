// jiff::Timestamp from a `SystemTime` (upstream timestamp.rs `new`,
// `from_duration` and `TryFrom<SystemTime>`, with signed_duration.rs
// `system_until`, `checked_neg` and `TryFrom<Duration>`).

use std::time::{Duration, SystemTime};

use crate::{
    error::{ConvertSystemTime, ErrorContext},
    jcore::Timestamp as JTimestamp,
    Error,
};

/// An instant in time: seconds and nanoseconds since the Unix epoch,
/// in years -9999 to 9999.
#[derive(Clone, Copy)]
pub struct Timestamp {
    dur: JTimestamp,
}

impl Timestamp {
    /// Upstream `Timestamp::new`: fails when the instant is out of range.
    fn new(second: i64, nanosecond: i32) -> Result<Timestamp, Error> {
        let dur = JTimestamp::new(second, nanosecond).map_err(Error::jcore_range)?;
        Ok(Timestamp { dur })
    }

    pub(crate) const fn to_jcore(&self) -> JTimestamp {
        self.dur
    }
}

impl TryFrom<SystemTime> for Timestamp {
    type Error = Error;

    fn try_from(system_time: SystemTime) -> Result<Timestamp, Error> {
        let unix_epoch = SystemTime::UNIX_EPOCH;
        let dur = SignedDuration::system_until(unix_epoch, system_time)?;
        Timestamp::from_duration(dur)
    }
}

impl Timestamp {
    fn from_duration(duration: SignedDuration) -> Result<Timestamp, Error> {
        Timestamp::new(duration.secs, duration.nanos)
    }
}

/// The part of upstream's `SignedDuration` the conversion uses.
struct SignedDuration {
    secs: i64,
    nanos: i32,
}

impl SignedDuration {
    const fn new_unchecked(secs: i64, nanos: i32) -> SignedDuration {
        debug_assert!(nanos <= 999_999_999);
        debug_assert!(nanos >= -999_999_999);
        debug_assert!(secs == 0 || nanos == 0 || secs.signum() == (nanos.signum() as i64));
        SignedDuration { secs, nanos }
    }

    /// Upstream `TryFrom<Duration> for SignedDuration`.
    fn try_from_duration(d: Duration) -> Result<SignedDuration, Error> {
        let secs = i64::try_from(d.as_secs()).map_err(|_| Error::bounds_signed_duration_seconds())?;
        // Guaranteed to succeed since 0<=nanos<=999,999,999.
        let nanos = i32::try_from(d.subsec_nanos()).unwrap();
        Ok(SignedDuration::new_unchecked(secs, nanos))
    }

    const fn checked_neg(self) -> Option<SignedDuration> {
        let Some(secs) = self.secs.checked_neg() else { return None };
        Some(SignedDuration::new_unchecked(
            secs,
            // Always OK because `-999_999_999 <= self.nanos <= 999_999_999`.
            -self.nanos,
        ))
    }

    fn system_until(time1: SystemTime, time2: SystemTime) -> Result<SignedDuration, Error> {
        match time2.duration_since(time1) {
            Ok(dur) => SignedDuration::try_from_duration(dur).context(ConvertSystemTime),
            Err(err) => {
                let dur = err.duration();
                let dur = SignedDuration::try_from_duration(dur).context(ConvertSystemTime)?;
                dur.checked_neg().ok_or_else(Error::bounds_signed_duration_seconds).context(ConvertSystemTime)
            }
        }
    }
}
