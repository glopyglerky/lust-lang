// `core::time::Duration` is lust std source (library/core/src/time.rs,
// W-STD-C), spliced as a hidden module that `std::time` and
// `core::time` paths walk into. Constructors, accessors, checked and
// saturating math, operators, float conversions, consts, and the exact
// Debug text (precision rounding with ties to even, `+`, width, fill,
// and alignment).
use core::time::Duration as CoreDuration;
use std::time;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(5);
const HALF: Duration = Duration::from_millis(500);
static TICK: Duration = Duration::from_micros(250);

fn show(d: Duration) {
    println!("{:?} secs={} ms={} us={} ns={} sub_ms={} sub_us={} sub_ns={}", d, d.as_secs(), d.as_millis(), d.as_micros(), d.as_nanos(), d.subsec_millis(), d.subsec_micros(), d.subsec_nanos());
}

fn main() {
    show(Duration::new(1, 999_999_999));
    show(Duration::new(3, 2_500_000_000));
    show(Duration::from_nanos(1_234_567_891));
    show(Duration::from_nanos_u128(98_765_432_101));
    show(Duration::from_mins(3));
    show(Duration::from_hours(2));
    show(TIMEOUT + HALF + TICK);
    println!("{:?} {:?} {:?}", Duration::ZERO, Duration::MAX, Duration::default());
    println!("{} {}", Duration::ZERO.is_zero(), TICK.is_zero());

    let a = Duration::from_millis(1500);
    let b = Duration::from_millis(700);
    println!("{:?} {:?} {:?} {:?}", a + b, a - b, a * 3, a / 4);
    println!("{:?} {:?}", 3 * b, b.abs_diff(a));
    println!("{:?} {:?}", a.checked_sub(b), b.checked_sub(a));
    println!("{:?} {:?}", a.checked_add(Duration::MAX), Duration::MAX.saturating_add(a));
    println!("{:?} {:?}", b.saturating_sub(a), a.saturating_mul(u32::MAX));
    println!("{:?} {:?} {:?}", a.checked_mul(2), a.checked_div(0), a.checked_div(7));
    let mut c = a;
    c += b;
    c -= Duration::from_nanos(3);
    c *= 5;
    c /= 3;
    println!("{:?}", c);

    println!("{} {} {}", a.as_secs_f64(), a.as_secs_f32(), a.div_duration_f64(b));
    println!("{}", b.div_duration_f32(a));
    println!("{:?} {:?} {:?}", Duration::from_secs_f64(2.5), Duration::from_secs_f32(0.75), Duration::from_secs_f64(1e-10));
    println!("{:?} {:?}", Duration::from_secs_f64(0.000_000_001_5), Duration::from_secs_f64(123456789.123456789));
    println!("{:?} {:?}", a.mul_f64(1.5), a.div_f32(4.0));
    println!("{:?}", Duration::try_from_secs_f64(-1.0));
    println!("{:?}", Duration::try_from_secs_f32(f32::NAN));
    match Duration::try_from_secs_f64(1e30) {
        Ok(d) => println!("ok {:?}", d),
        Err(e) => println!("err {} / {:?}", e, e),
    }
    println!("{:?}", Duration::try_from_secs_f64(0.999_999_999_6));

    // Debug text: units, precision, rounding, and padding.
    for d in [Duration::from_nanos(7), Duration::from_nanos(1_500), Duration::from_micros(2_345), Duration::from_millis(12_345), Duration::new(0, 999_999_999)] {
        println!("[{:?}] [{:.0?}] [{:.1?}] [{:.3?}] [{:.12?}] [{:+?}] [{:#?}]", d, d, d, d, d, d, d);
        println!("[{:12?}] [{:<12?}] [{:>12?}] [{:^12?}] [{:*^13.2?}] [{:>+10.1?}] [{:2?}]", d, d, d, d, d, d, d);
    }
    println!("{:.0?} {:.0?} {:.0?} {:.0?}", Duration::from_millis(1500), Duration::from_millis(2500), Duration::from_millis(3500), Duration::new(4, 500_000_001));
    println!("{:.2?} {:.2?}", Duration::new(u64::MAX, 999_999_999), Duration::from_nanos(9_999));

    // Comparison, hashing order, sorting, and the other path forms.
    let mut v = vec![Duration::from_secs(2), Duration::from_millis(10), Duration::ZERO, Duration::from_nanos(999)];
    for i in 0..v.len() {
        for j in 0..v.len() - 1 - i {
            if v[j] > v[j + 1] {
                v.swap(j, j + 1);
            }
        }
    }
    println!("{:?}", v);
    println!("{} {} {:?}", a > b, a == Duration::from_micros(1_500_000), a.max(b));
    let x: CoreDuration = time::Duration::from_secs(1);
    let y: std::time::Duration = core::time::Duration::from_secs(1);
    println!("{}", x == y);
    println!("{:?}", Some(Duration::from_millis(3)));
}
