//! Data services: gzip, xxh3, jiff time zones, and BigInt values.
//!
//! Every call here is whole-buffer or whole-value, matching the
//! ts-rust call sites. Streaming state exists only where ts-rust
//! streams (Xxh3).

use std::ffi::c_int;
use std::io::{Read, Write};

use crate::{Handles, ShimBuf, ShimErr, ShimResult, buf_put, bytes, put, status};

// ---------------------------------------------------------------------
// gzip (flate2)
// ---------------------------------------------------------------------

/// One gzip member at a flate2 level (pprof uses fast, level 1).
/// A level over 9 is an argument error, where flate2 would clamp by
/// its Compression::new contract of 0 to 9.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_gzip_encode(input: *const u8, in_len: usize, level: u32, out: *mut ShimBuf) -> c_int {
    status(|| {
        if level > 9 {
            return Err(ShimErr::invalid("level"));
        }
        let data = unsafe { bytes(input, in_len, "input") }?;
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
        enc.write_all(data)?;
        let done = enc.finish()?;
        unsafe { buf_put(out, done) }
    })
}

/// Multi-member gzip decode to the end (locale data load). A bad
/// header or a cut member is an error with the flate2 text, where
/// ts-rust panics with that text included.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_gzip_decode_multi(input: *const u8, in_len: usize, out: *mut ShimBuf) -> c_int {
    status(|| {
        let data = unsafe { bytes(input, in_len, "input") }?;
        let mut dec = flate2::read::MultiGzDecoder::new(data);
        let mut done = Vec::new();
        dec.read_to_end(&mut done)?;
        unsafe { buf_put(out, done) }
    })
}

// ---------------------------------------------------------------------
// xxh3 (xxhash-rust)
// ---------------------------------------------------------------------

/// The one-shot 64-bit digest.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_64(input: *const u8, in_len: usize, out: *mut u64) -> c_int {
    status(|| {
        let data = unsafe { bytes(input, in_len, "input") }?;
        unsafe { put(out, xxhash_rust::xxh3::xxh3_64(data), "out") }
    })
}

/// The one-shot 128-bit digest as low and high u64 words.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_128(input: *const u8, in_len: usize, out_lo: *mut u64, out_hi: *mut u64) -> c_int {
    status(|| {
        let data = unsafe { bytes(input, in_len, "input") }?;
        let v = xxhash_rust::xxh3::xxh3_128(data);
        unsafe { put(out_lo, v as u64, "out_lo") }?;
        unsafe { put(out_hi, (v >> 64) as u64, "out_hi") }
    })
}

static HASHERS: Handles<xxhash_rust::xxh3::Xxh3> = Handles::new();

/// A new streaming hasher (Xxh3::new, default secret, as in ts-rust).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_new(out_handle: *mut u64) -> c_int {
    status(|| {
        let id = HASHERS.insert(xxhash_rust::xxh3::Xxh3::new());
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// Feeds one chunk to a streaming hasher.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_update(handle: u64, input: *const u8, in_len: usize) -> c_int {
    status(|| {
        let data = unsafe { bytes(input, in_len, "input") }?;
        HASHERS.with(handle, "xxh3 handle", |h| {
            h.update(data);
            Ok(())
        })
    })
}

/// The streaming 64-bit digest so far. The hasher stays usable.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_digest64(handle: u64, out: *mut u64) -> c_int {
    status(|| {
        let v = HASHERS.with(handle, "xxh3 handle", |h| Ok(h.digest()))?;
        unsafe { put(out, v, "out") }
    })
}

/// The streaming 128-bit digest so far, as low and high words.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_digest128(handle: u64, out_lo: *mut u64, out_hi: *mut u64) -> c_int {
    status(|| {
        let v = HASHERS.with(handle, "xxh3 handle", |h| Ok(h.digest128()))?;
        unsafe { put(out_lo, v as u64, "out_lo") }?;
        unsafe { put(out_hi, (v >> 64) as u64, "out_hi") }
    })
}

/// Drops a streaming hasher.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_xxh3_free(handle: u64) -> c_int {
    status(|| {
        HASHERS.remove(handle, "xxh3 handle")?;
        Ok(())
    })
}

// ---------------------------------------------------------------------
// Time zones and civil time (jiff)
// ---------------------------------------------------------------------

/// The C record for one civil date-time. Must match LustShimDateTime.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CDateTime {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub subsec_nanos: i32,
}

static ZONES: Handles<jiff::tz::TimeZone> = Handles::new();

/// The civil time of a Unix timestamp in a zone. A timestamp outside
/// the jiff range is an error, where ts-rust marks the case unported.
fn civil_of(tz: &jiff::tz::TimeZone, unix_sec: i64, nanos: i32) -> ShimResult<jiff::civil::DateTime> {
    let ts = jiff::Timestamp::new(unix_sec, nanos).map_err(|e| ShimErr::msg(format!("timestamp out of range: {e}")))?;
    Ok(tz.to_datetime(ts))
}

/// Runs f with the zone for one handle.
fn with_zone<R>(handle: u64, f: impl FnOnce(&jiff::tz::TimeZone) -> ShimResult<R>) -> ShimResult<R> {
    ZONES.with(handle, "time zone handle", |tz| f(tz))
}

/// A handle for UTC.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_tz_utc(out_handle: *mut u64) -> c_int {
    status(|| {
        let id = ZONES.insert(jiff::tz::TimeZone::UTC);
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// A zone from raw TZif bytes (jiff TimeZone::tzif).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_tz_from_tzif(
    name: *const u8,
    name_len: usize,
    data: *const u8,
    data_len: usize,
    out_handle: *mut u64,
) -> c_int {
    status(|| {
        let name = unsafe { bytes(name, name_len, "name") }?;
        let data = unsafe { bytes(data, data_len, "data") }?;
        let tz = jiff::tz::TimeZone::tzif(&String::from_utf8_lossy(name), data).map_err(|e| ShimErr::msg(format!("bad TZif data: {e}")))?;
        let id = ZONES.insert(tz);
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// Reads one zone file (Go loadTzinfo directory case). Files over the
/// Go readFile cap (10 MiB) do not count.
fn load_tzinfo(name: &[u8], source: &str) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let mut path = Vec::new();
    if !source.is_empty() {
        path.extend_from_slice(source.as_bytes());
        path.push(b'/');
    }
    path.extend_from_slice(name);
    let data = std::fs::read(std::ffi::OsStr::from_bytes(&path)).ok()?;
    (data.len() <= (10 << 20)).then_some(data)
}

/// Tries each source for a zone name (Go loadLocation, jiff TZif).
fn load_location(name: &[u8], sources: &[&str]) -> Option<jiff::tz::TimeZone> {
    for source in sources {
        if let Some(data) = load_tzinfo(name, source)
            && let Ok(z) = jiff::tz::TimeZone::tzif(&String::from_utf8_lossy(name), &data)
        {
            return Some(z);
        }
    }
    None
}

/// The zone Go time.Local picks on Unix, resolved the way ts-rust
/// execute/tsc/diagnostics.rs init_local resolves it. Reads TZ at
/// call time; callers call once and keep the handle.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_tz_local(out_handle: *mut u64) -> c_int {
    status(|| {
        const SOURCES: &[&str] = &["/usr/share/zoneinfo/", "/usr/share/lib/zoneinfo/", "/usr/lib/locale/TZ/", "/etc/zoneinfo"];
        let tz_env = std::env::var_os("TZ");
        let mut zone = None;
        match tz_env.as_ref().map(|v| v.as_encoded_bytes()) {
            None => zone = load_location(b"localtime", &["/etc"]),
            Some(tz) if !tz.is_empty() => {
                let tz = tz.strip_prefix(b":").unwrap_or(tz);
                if tz.first() == Some(&b'/') {
                    zone = load_location(tz, &[""]);
                } else if !tz.is_empty() && tz != b"UTC" {
                    zone = load_location(tz, SOURCES);
                }
            }
            Some(_) => {}
        }
        let id = ZONES.insert(zone.unwrap_or(jiff::tz::TimeZone::UTC));
        unsafe { put(out_handle, id, "out_handle") }
    })
}

/// Drops a time zone handle.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_tz_free(handle: u64) -> c_int {
    status(|| {
        ZONES.remove(handle, "time zone handle")?;
        Ok(())
    })
}

/// Civil fields of a Unix timestamp in the zone.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_tz_to_datetime(tz: u64, unix_sec: i64, nanos: i32, out: *mut CDateTime) -> c_int {
    status(|| {
        let dt = with_zone(tz, |z| civil_of(z, unix_sec, nanos))?;
        let v = CDateTime {
            year: i32::from(dt.year()),
            month: i32::from(dt.month()),
            day: i32::from(dt.day()),
            hour: i32::from(dt.hour()),
            minute: i32::from(dt.minute()),
            second: i32::from(dt.second()),
            subsec_nanos: dt.subsec_nanosecond(),
        };
        unsafe { put(out, v, "out") }
    })
}

/// The two clocks ts-rust prints, formatted as there. Kind 0 is the
/// status clock (Go "03:04:05 PM"), kind 1 the log clock with millis.
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_jiff_format_clock(tz: u64, unix_sec: i64, nanos: i32, kind: u32, out: *mut ShimBuf) -> c_int {
    status(|| {
        let dt = with_zone(tz, |z| civil_of(z, unix_sec, nanos))?;
        let text = match kind {
            0 => {
                let hour = dt.hour();
                let mut hr = hour % 12;
                if hr == 0 {
                    hr = 12;
                }
                let pm = if hour >= 12 { "PM" } else { "AM" };
                format!("{hr:02}:{:02}:{:02} {pm}", dt.minute(), dt.second())
            }
            1 => format!("{:02}:{:02}:{:02}.{:03}", dt.hour(), dt.minute(), dt.second(), dt.subsec_nanosecond() / 1_000_000),
            _ => return Err(ShimErr::invalid("clock kind")),
        };
        unsafe { buf_put(out, text.into_bytes()) }
    })
}

// ---------------------------------------------------------------------
// BigInt whole-value calls (num-bigint, num-traits)
// ---------------------------------------------------------------------
//
// jsnum.rs makes exactly three BigInt calls, each whole-value:
// parse then to f64, parse then print decimal, and i64 pow then to
// f64. No general arithmetic crosses, so no BigInt handle exists.
// The f64 to_i64 and to_u32 calls in jsnum are num-traits on f64,
// which is plain float cast rules and needs no shim call.

/// BigInt::parse_bytes, then num-traits to_f64. A None at either step
/// gives out_has 0 with an OK status (the ts-rust Option path).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_bigint_parse_to_f64(
    digits: *const u8,
    digits_len: usize,
    radix: u32,
    out: *mut f64,
    out_has: *mut i32,
) -> c_int {
    use num_traits::ToPrimitive;
    status(|| {
        let digits = unsafe { bytes(digits, digits_len, "digits") }?;
        let value = num_bigint::BigInt::parse_bytes(digits, radix).and_then(|v| v.to_f64());
        match value {
            Some(v) => {
                unsafe { put(out, v, "out") }?;
                unsafe { put(out_has, 1, "out_has") }
            }
            None => unsafe { put(out_has, 0, "out_has") },
        }
    })
}

/// BigInt::parse_bytes, then the decimal Display text (the
/// parse_pseudo_big_int path, minus its panic on a bad literal).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_bigint_parse_to_decimal(
    digits: *const u8,
    digits_len: usize,
    radix: u32,
    out: *mut ShimBuf,
    out_has: *mut i32,
) -> c_int {
    status(|| {
        let digits = unsafe { bytes(digits, digits_len, "digits") }?;
        match num_bigint::BigInt::parse_bytes(digits, radix) {
            Some(v) => {
                unsafe { buf_put(out, v.to_string().into_bytes()) }?;
                unsafe { put(out_has, 1, "out_has") }
            }
            None => unsafe { put(out_has, 0, "out_has") },
        }
    })
}

/// BigInt::from(i64).pow(u32), then to_f64 (Number::exponentiate).
#[unsafe(no_mangle)]
pub extern "C" fn lust_shim_bigint_pow_i64_to_f64(base: i64, power: u32, out: *mut f64, out_has: *mut i32) -> c_int {
    use num_traits::ToPrimitive;
    status(|| {
        let value = num_bigint::BigInt::from(base).pow(power).to_f64();
        match value {
            Some(v) => {
                unsafe { put(out, v, "out") }?;
                unsafe { put(out_has, 1, "out_has") }
            }
            None => unsafe { put(out_has, 0, "out_has") },
        }
    })
}
