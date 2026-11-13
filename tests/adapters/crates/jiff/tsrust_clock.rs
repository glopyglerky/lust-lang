// jiff adapter: ts-rust's shape (execute/tsc/diagnostics.rs,
// project/logging/logger.rs): a `static OnceLock<TimeZone>`, a fallback
// to `TimeZone::UTC`, and the 12-hour and millisecond clocks.

use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

static LOCAL_LOC: OnceLock<jiff::tz::TimeZone> = OnceLock::new();

fn init_local() -> jiff::tz::TimeZone {
    let data: &[u8] = b"not tzif";
    if let Ok(z) = jiff::tz::TimeZone::tzif("Bad/Zone", data) {
        return z;
    }
    jiff::tz::TimeZone::UTC
}

fn local_location() -> &'static jiff::tz::TimeZone {
    LOCAL_LOC.get_or_init(init_local)
}

fn format_status_time(now: SystemTime) -> String {
    let Ok(timestamp) = jiff::Timestamp::try_from(now) else {
        return "unported".to_string();
    };
    let datetime = local_location().to_datetime(timestamp);
    let hour = datetime.hour();
    let mut hr = hour % 12;
    if hr == 0 {
        hr = 12;
    }
    let pm = if hour >= 12 { "PM" } else { "AM" };
    format!("{hr:02}:{:02}:{:02} {pm}", datetime.minute(), datetime.second())
}

fn format_clock_millis_in(t: SystemTime, zone: &jiff::tz::TimeZone) -> String {
    let Ok(timestamp) = jiff::Timestamp::try_from(t) else {
        return "unported".to_string();
    };
    let datetime = zone.to_datetime(timestamp);
    format!("{:02}:{:02}:{:02}.{:03}", datetime.hour(), datetime.minute(), datetime.second(), datetime.subsec_nanosecond() / 1_000_000)
}

fn main() {
    for secs in [0u64, 43_199, 43_200, 86_399, 1_700_000_000, 253_402_300_799, 253_402_300_800] {
        for nanos in [0u32, 999_999, 1_000_000, 999_999_999] {
            let t = SystemTime::UNIX_EPOCH + Duration::new(secs, nanos);
            println!("{secs}.{nanos} [{}] [{}]", format_status_time(t), format_clock_millis_in(t, local_location()));
        }
    }
    let shared = std::thread::spawn(|| format_status_time(SystemTime::UNIX_EPOCH + Duration::from_secs(3600))).join().unwrap();
    println!("thread {shared}");
}
