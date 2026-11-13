// jiff adapter: upstream's errors, through Display, Debug, and {:#?}:
// TZif parse errors, timestamp range errors, and SystemTime conversion
// errors.

use std::time::{Duration, SystemTime};

fn main() {
    let mut valid = Vec::new();
    valid.extend_from_slice(b"TZif2");
    valid.extend_from_slice(&[0u8; 15]);
    for c in [0u32, 0, 0, 0, 1, 4] {
        valid.extend_from_slice(&c.to_be_bytes());
    }
    valid.extend_from_slice(&0i32.to_be_bytes());
    valid.extend_from_slice(&[0, 0]);
    valid.extend_from_slice(b"UTC\0");
    let v1 = valid.clone();
    valid.extend_from_slice(&v1);
    valid.extend_from_slice(b"\nUTC0\n");
    let mut cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("short", b"TZif2".to_vec()),
        ("magic", b"TZiq2xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_vec()),
        ("valid", valid.clone()),
    ];
    let mut no_footer = valid.clone();
    no_footer.truncate(valid.len() - 6);
    cases.push(("no footer", no_footer));
    let mut bad_footer = valid.clone();
    bad_footer.truncate(valid.len() - 5);
    bad_footer.extend_from_slice(b"EST5EDT,M13.2.0,M11.1.0\n");
    cases.push(("bad footer", bad_footer));
    let mut mismatch = valid.clone();
    mismatch.truncate(valid.len() - 5);
    mismatch.extend_from_slice(b"EST5\n");
    cases.push(("footer mismatch", mismatch));
    let mut big_type = valid.clone();
    big_type[44 + 4..44 + 6].copy_from_slice(&[0xff, 0xff]);
    cases.push(("type offset", big_type));
    for (name, data) in &cases {
        match jiff::tz::TimeZone::tzif("Test/Zone", data) {
            Ok(_) => println!("{name}: ok"),
            Err(e) => println!("{name}: {e}\n{e:?}\n{e:#?}"),
        }
    }
    let times = [
        SystemTime::UNIX_EPOCH + Duration::from_secs(253402300800),
        SystemTime::UNIX_EPOCH - Duration::from_secs(377705116801),
        SystemTime::UNIX_EPOCH - Duration::from_secs(1 << 63),
        SystemTime::UNIX_EPOCH - Duration::new((1 << 63) - 1, 999_999_999),
    ];
    for t in times {
        match jiff::Timestamp::try_from(t) {
            Ok(_) => println!("ok"),
            Err(e) => {
                let c = e.clone();
                let as_std: &dyn std::error::Error = &c;
                println!("{e}\n{e:?}\n{e:#?}\n{}", as_std);
            }
        }
    }
}
