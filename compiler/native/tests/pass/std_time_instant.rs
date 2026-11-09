// `std::time::{Instant, SystemTime, UNIX_EPOCH, SystemTimeError}` are
// lust std source (library/std/src/time.rs, W-STD-C) over the
// `clock_now_raw` and `system_time_raw` intrinsics. Clock values differ
// per run, so this prints only facts that hold on every run: ordering,
// round trips through Duration math, saturation, and the exact text of
// fixed times and errors.
use std::time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

fn since_epoch(t: SystemTime) -> Result<Duration, SystemTimeError> {
    t.duration_since(UNIX_EPOCH)
}

fn main() {
    let start = Instant::now();
    let mut spin = 0u64;
    for i in 0..10_000u64 {
        spin = spin.wrapping_add(i * i);
    }
    let later = Instant::now();
    println!("spin {}", spin);
    println!("ordered {} {}", later >= start, start <= later);
    let el = start.elapsed();
    println!("elapsed small {}", el < Duration::from_secs(60));
    println!("since {}", later.duration_since(start) <= later.elapsed() + later.duration_since(start));
    // Earlier minus later saturates to zero.
    println!("saturating {:?} {:?}", start.duration_since(later + Duration::from_secs(1)), start.saturating_duration_since(later + Duration::from_secs(1)));
    println!("checked {:?}", start.checked_duration_since(start + Duration::from_nanos(1)));
    let shifted = start + Duration::from_millis(1500);
    println!("shift {:?} {:?}", shifted - start, shifted.duration_since(start));
    let mut moving = start;
    moving += Duration::from_secs(3);
    moving -= Duration::from_millis(250);
    println!("moved {:?}", moving - start);
    println!("checked add {}", start.checked_add(Duration::MAX).is_none());
    match start.checked_sub(Duration::from_secs(1)) {
        Some(t) => println!("checked sub {:?}", start - t),
        None => println!("checked sub none"),
    }
    println!("eq {} {}", start == start, start != shifted);

    let now = SystemTime::now();
    match since_epoch(now) {
        Ok(d) => println!("after 2020 {}", d.as_secs() > 1_577_836_800),
        Err(e) => println!("before epoch {}", e),
    }
    println!("epoch {:?} {:?}", UNIX_EPOCH, SystemTime::UNIX_EPOCH);
    let t = UNIX_EPOCH + Duration::new(1_700_000_000, 123);
    println!("fixed {:?} {:?}", t, since_epoch(t));
    let before = UNIX_EPOCH - Duration::from_millis(1500);
    println!("before {:?}", before);
    match since_epoch(before) {
        Ok(d) => println!("ok {:?}", d),
        Err(e) => println!("err {} {:?} {:?}", e, e.duration(), e),
    }
    if let Err(e) = before.duration_since(t) {
        println!("{:?}", e.duration());
    }
    let mut st = t;
    st += Duration::from_secs(10);
    st -= Duration::from_nanos(124);
    println!("{:?} {:?}", st, st.checked_sub(Duration::from_secs(u64::MAX)));
    println!("{:?}", t.checked_add(Duration::from_secs(5)));
    if let Ok(d) = SystemTime::now().elapsed() {
        println!("{}", d < Duration::from_secs(60));
    }
    println!("{} {}", t > before, before < UNIX_EPOCH);
}
