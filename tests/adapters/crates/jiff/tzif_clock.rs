// jiff adapter: civil clock times from TZif data built here (v1, v2, v3
// and v4 forms; POSIX footers with and without DST; leap seconds), at
// transition edges, in POSIX-rule years, and at the range edges.

use std::time::{Duration, SystemTime};

use jiff::tz::TimeZone;

fn tzif(version: u8, times: &[i64], idxs: &[u8], types: &[(i32, u8, u8)], chars: &[u8], leaps: &[(i64, i32)], footer: Option<&str>) -> Vec<u8> {
    let mut out = Vec::new();
    let header = |out: &mut Vec<u8>, n: usize| {
        out.extend_from_slice(b"TZif");
        out.push(version);
        out.extend_from_slice(&[0u8; 15]);
        for c in [0, 0, leaps.len(), n, types.len(), chars.len()] {
            out.extend_from_slice(&(c as u32).to_be_bytes());
        }
    };
    let body = |out: &mut Vec<u8>, wide: bool, times: &[i64], idxs: &[u8]| {
        for &t in times {
            if wide { out.extend_from_slice(&t.to_be_bytes()) } else { out.extend_from_slice(&(t as i32).to_be_bytes()) }
        }
        out.extend_from_slice(idxs);
        for &(off, dst, d) in types {
            out.extend_from_slice(&off.to_be_bytes());
            out.push(dst);
            out.push(d);
        }
        out.extend_from_slice(chars);
        for &(t, c) in leaps {
            if wide { out.extend_from_slice(&t.to_be_bytes()) } else { out.extend_from_slice(&(t as i32).to_be_bytes()) }
            out.extend_from_slice(&c.to_be_bytes());
        }
    };
    let narrow: Vec<usize> = (0..times.len()).filter(|&k| times[k] >= i32::MIN as i64 && times[k] <= i32::MAX as i64).collect();
    let t32: Vec<i64> = narrow.iter().map(|&k| times[k]).collect();
    let i32s: Vec<u8> = narrow.iter().map(|&k| idxs[k]).collect();
    header(&mut out, t32.len());
    body(&mut out, false, &t32, &i32s);
    if version != 0 {
        header(&mut out, times.len());
        body(&mut out, true, times, idxs);
        if let Some(f) = footer {
            out.push(b'\n');
            out.extend_from_slice(f.as_bytes());
            out.push(b'\n');
        }
    }
    out
}

fn at(secs: i64, nanos: u32) -> SystemTime {
    if secs >= 0 {
        SystemTime::UNIX_EPOCH + Duration::new(secs as u64, nanos)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_secs(secs.unsigned_abs()) + Duration::new(0, nanos)
    }
}

fn show(name: &str, zone: &TimeZone, secs: i64, nanos: u32) {
    match jiff::Timestamp::try_from(at(secs, nanos)) {
        Ok(ts) => {
            let d = zone.to_datetime(ts);
            println!("{name} {secs}.{nanos:09} {:02}:{:02}:{:02}.{:09}", d.hour(), d.minute(), d.second(), d.subsec_nanosecond());
        }
        Err(e) => println!("{name} {secs}.{nanos:09} err {e}"),
    }
}

fn main() {
    // New York style: LMT, then EST/EDT, then the US rule in the footer.
    let chars = b"LMT\0EST\0EDT\0";
    let types = [(-17762, 0, 0), (-18000, 0, 4), (-14400, 1, 8)];
    let times = [-2717650800, 1678604400, 1699164000, 1710054000, 1730613600];
    let idxs = [1, 2, 1, 2, 1];
    let mut zones: Vec<(String, TimeZone)> = Vec::new();
    for v in [0u8, b'2', b'3', b'4'] {
        let data = tzif(v, &times, &idxs, &types, chars, &[], Some("EST5EDT,M3.2.0,M11.1.0"));
        zones.push((format!("ny-v{v}"), TimeZone::tzif("America/New_York", &data).unwrap()));
    }
    // A half-hour zone with a southern rule and a 24h+ transition time.
    let chars = b"ACST\0ACDT\0";
    let data = tzif(b'3', &[1680366600], &[0], &[(34200, 0, 0), (37800, 1, 5)], chars, &[], Some("ACST-9:30ACDT,M10.1.0,M4.1.0/3"));
    zones.push(("adelaide".to_string(), TimeZone::tzif("Australia/Adelaide", &data).unwrap()));
    let data = tzif(b'3', &[], &[], &[(7200, 0, 0), (10800, 1, 4)], b"IST\0IDT\0", &[], Some("IST-2IDT,M3.4.4/26,M10.5.0"));
    zones.push(("jerusalem".to_string(), TimeZone::tzif("Asia/Jerusalem", &data).unwrap()));
    // No transitions, no footer: type 0 for every instant.
    let data = tzif(b'2', &[], &[], &[(19800, 0, 0)], b"IST\0", &[], Some(""));
    zones.push(("kolkata".to_string(), TimeZone::tzif("Asia/Kolkata", &data).unwrap()));
    // Leap second records are read and checked, not applied.
    let data = tzif(b'2', &[78796800], &[0], &[(0, 0, 0)], b"UTC\0", &[(78796800, 1), (94694401, 2)], Some("UTC0"));
    zones.push(("leaps".to_string(), TimeZone::tzif("right/UTC", &data).unwrap()));
    zones.push(("utc".to_string(), TimeZone::UTC));
    let probes: &[(i64, u32)] = &[
        (-377705116800, 0),
        (-377705116801, 0),
        (-2717650801, 999_999_999),
        (-2717650800, 0),
        (-1, 0),
        (0, 0),
        (1678604399, 999_999_999),
        (1678604400, 0),
        (1699163999, 0),
        (1699164000, 1),
        (1710054000, 0),
        (1730613599, 999_999_999),
        (1730613600, 0),
        (1741503600, 0),
        (1762063200, 0),
        (1696089600, 0),
        (1712419200, 0),
        (4102444800, 123_456_789),
        (253402300799, 999_999_999),
        (253402300800, 0),
    ];
    for (name, zone) in &zones {
        let copy = zone.clone();
        for &(s, n) in probes {
            show(name, &copy, s, n);
        }
    }
}
