//! Rust oracle for the shim C test.
//!
//! It does the probe operations directly with the same crates the
//! shim wraps, with the same baked inputs as tests/test_shim.c, and
//! prints one `ORACLE key=value` line per result. test.sh diffs
//! these lines against the ORACLE lines from the C test, which got
//! its values through liblust_shim.so. A mismatch means the C ABI
//! changed the result of the real crate call.
//!
//! Args: <tzif-file> <zone-name> <stat-file>

use std::io::{Read, Write};

/// The shared probe pattern. test_shim.c builds it with the same rule.
fn pattern(n: usize) -> Vec<u8> {
    (0..n).map(|i| ((i * 31 + 7) % 256) as u8).collect()
}

/// The civil fields of a timestamp in a zone, one text form for diff.
fn dt_text(tz: &jiff::tz::TimeZone, sec: i64, nanos: i32) -> String {
    let ts = jiff::Timestamp::new(sec, nanos).expect("probe timestamp in range");
    let dt = tz.to_datetime(ts);
    format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}.{}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.subsec_nanosecond()
    )
}

/// The local zone, resolved as ts-rust init_local does. Written here
/// against jiff directly, separate from the shim copy.
fn local_zone() -> jiff::tz::TimeZone {
    fn load(name: &[u8], sources: &[&str]) -> Option<jiff::tz::TimeZone> {
        use std::os::unix::ffi::OsStrExt;
        for source in sources {
            let mut path = Vec::new();
            if !source.is_empty() {
                path.extend_from_slice(source.as_bytes());
                path.push(b'/');
            }
            path.extend_from_slice(name);
            if let Ok(data) = std::fs::read(std::ffi::OsStr::from_bytes(&path))
                && let Ok(z) = jiff::tz::TimeZone::tzif(&String::from_utf8_lossy(name), &data)
            {
                return Some(z);
            }
        }
        None
    }
    const SOURCES: &[&str] = &["/usr/share/zoneinfo/", "/usr/share/lib/zoneinfo/", "/usr/lib/locale/TZ/", "/etc/zoneinfo"];
    let tz_env = std::env::var_os("TZ");
    let zone = match tz_env.as_ref().map(|v| v.as_encoded_bytes()) {
        None => load(b"localtime", &["/etc"]),
        Some(tz) if !tz.is_empty() => {
            let tz = tz.strip_prefix(b":").unwrap_or(tz);
            if tz.first() == Some(&b'/') {
                load(tz, &[""])
            } else if !tz.is_empty() && tz != b"UTC" {
                load(tz, SOURCES)
            } else {
                None
            }
        }
        Some(_) => None,
    };
    zone.unwrap_or(jiff::tz::TimeZone::UTC)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 4, "usage: oracle <tzif-file> <zone-name> <stat-file>");
    let out = |k: &str, v: &dyn std::fmt::Display| println!("ORACLE {k}={v}");

    // xxhash-rust, one-shot and streaming.
    let pat = pattern(10_000);
    out("xx64_empty", &xxhash_rust::xxh3::xxh3_64(b""));
    out("xx64_hello", &xxhash_rust::xxh3::xxh3_64(b"hello lust shim"));
    let h128 = xxhash_rust::xxh3::xxh3_128(b"hello lust shim");
    out("xx128_hello_lo", &(h128 as u64));
    out("xx128_hello_hi", &((h128 >> 64) as u64));
    out("xx64_pattern", &xxhash_rust::xxh3::xxh3_64(&pat));
    let p128 = xxhash_rust::xxh3::xxh3_128(&pat);
    out("xx128_pattern_lo", &(p128 as u64));
    out("xx128_pattern_hi", &((p128 >> 64) as u64));
    let mut stream = xxhash_rust::xxh3::Xxh3::new();
    stream.update(&pat[..3000]);
    stream.update(&pat[3000..3001]);
    stream.update(&pat[3001..]);
    out("xx_stream64", &stream.digest());
    let s128 = stream.digest128();
    out("xx_stream128_lo", &(s128 as u64));
    out("xx_stream128_hi", &((s128 >> 64) as u64));

    // flate2 gzip at the pprof level (fast), plus a decode round trip.
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(&pat).unwrap();
    let gz = enc.finish().unwrap();
    out("gzip_len", &gz.len());
    out("gzip_xx64", &xxhash_rust::xxh3::xxh3_64(&gz));
    let mut back = Vec::new();
    flate2::read::MultiGzDecoder::new(&gz[..]).read_to_end(&mut back).unwrap();
    out("gzip_roundtrip_xx64", &xxhash_rust::xxh3::xxh3_64(&back));

    // num-bigint whole-value calls.
    use num_traits::ToPrimitive;
    let f = num_bigint::BigInt::parse_bytes(b"123456789abcdef", 16).and_then(|v| v.to_f64()).unwrap();
    out("bigint_hex_f64_bits", &f.to_bits());
    let dec = num_bigint::BigInt::parse_bytes(b"ffffffffffffffff", 16).unwrap();
    out("bigint_dec", &dec.to_string());
    let pw = num_bigint::BigInt::from(3i64).pow(40).to_f64().unwrap();
    out("bigint_pow_bits", &pw.to_bits());
    let bad: i32 = i32::from(num_bigint::BigInt::parse_bytes(b"12x", 10).is_some());
    out("bigint_bad_has", &bad);

    // jiff zone from the TZif file arg, and the two clocks.
    let zone_data = std::fs::read(&args[1]).unwrap();
    let tz = jiff::tz::TimeZone::tzif(&args[2], &zone_data).unwrap();
    out("tz_dt1", &dt_text(&tz, 1_709_651_045, 123_456_789));
    out("tz_dt_neg", &dt_text(&tz, -1, 0));
    let ts = jiff::Timestamp::new(1_709_651_045, 123_456_789).unwrap();
    let dt = tz.to_datetime(ts);
    let hour = dt.hour();
    let mut hr = hour % 12;
    if hr == 0 {
        hr = 12;
    }
    let pm = if hour >= 12 { "PM" } else { "AM" };
    out("tz_clock_status", &format!("{hr:02}:{:02}:{:02} {pm}", dt.minute(), dt.second()));
    out("tz_clock_millis", &format!("{:02}:{:02}:{:02}.{:03}", dt.hour(), dt.minute(), dt.second(), dt.subsec_nanosecond() / 1_000_000));
    out("tz_local_dt1", &dt_text(&local_zone(), 1_709_651_045, 123_456_789));

    // rustix stat family on the stat file arg.
    let fd = rustix::fs::open(&args[3], rustix::fs::OFlags::RDONLY, rustix::fs::Mode::empty()).unwrap();
    let st = rustix::fs::fstat(&fd).unwrap();
    out("stat_size", &st.st_size);
    out("stat_mode", &st.st_mode);
    out("stat_ino", &st.st_ino);
    out("stat_mtime", &st.st_mtime);
    let sx = rustix::fs::statx(
        rustix::fs::CWD,
        &*args[3],
        rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
        rustix::fs::StatxFlags::TYPE | rustix::fs::StatxFlags::MTIME,
    )
    .unwrap();
    out("statx_mode", &u32::from(sx.stx_mode));
    out("statx_mtime", &sx.stx_mtime.tv_sec);
}
