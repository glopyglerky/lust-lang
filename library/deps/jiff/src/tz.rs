// jiff::tz: `TimeZone` for UTC and for TZif data (upstream
// tz/timezone.rs). Upstream's `Repr` is a tagged pointer with six forms;
// the two this surface can make are an enum here. A TZif zone is shared
// through an `Arc`, as upstream's `ARC_TZIF` form is.

use std::sync::Arc;

use crate::{
    civil::DateTime,
    jcore::tz::{tzif, Offset, TimeZoneId},
    Error, Timestamp,
};

/// A time zone: UTC, or a zone read from TZif data.
#[derive(Clone)]
pub struct TimeZone {
    repr: Repr,
}

#[derive(Clone)]
enum Repr {
    Utc,
    ArcTzif(Arc<tzif::MaybeNamedTimeZone>),
}

impl TimeZone {
    /// The UTC time zone. The offset is always zero.
    pub const UTC: TimeZone = TimeZone { repr: Repr::Utc };

    /// A time zone from TZif data (RFC 8536, versions 1 to 4), named
    /// `name`. Fails, with upstream's error, when `data` is not valid
    /// TZif.
    pub fn tzif(name: &str, data: &[u8]) -> Result<TimeZone, Error> {
        let name = TimeZoneId::new_or_heap(name);
        let tzif = tzif::TimeZone::parse(data).map_err(Error::jcore_tzif_parse)?.into_named(name);
        let repr = Repr::ArcTzif(Arc::new(tzif));
        Ok(TimeZone { repr })
    }

    /// The civil date and clock time of `timestamp` in this zone.
    pub fn to_datetime(&self, timestamp: Timestamp) -> DateTime {
        DateTime::from_jcore(self.to_offset(timestamp).to_datetime(timestamp.to_jcore()))
    }

    /// Upstream `to_offset` (as jcore's `Offset`; upstream wraps it and
    /// unwraps it again in `Offset::to_datetime`).
    fn to_offset(&self, timestamp: Timestamp) -> Offset {
        match self.repr {
            Repr::Utc => Offset::UTC,
            Repr::ArcTzif(ref tzif) => tzif.tz().to_offset(timestamp.to_jcore()),
        }
    }
}
