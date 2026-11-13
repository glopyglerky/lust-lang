// ryu-js 1.0.2 Buffer::format on values at every formatting boundary:
// integers up to 1e21, the 1e-6 and 1e-7 switch, subnormals, extremes,
// signed zero, NaN, and infinities, plus the buffer reused and cloned.
fn main() {
    let mut b = ryu_js::Buffer::new();
    let values = [
        0.0, -0.0, 1.0, -1.0, 0.1, 0.5, 1.5, 100.0, 123456789.0, 1e20, 1e21, 1.5e21, 123e19, 1e-6, 1.5e-6,
        1e-7, 1.2345e-7, 0.000001234, 5e-324, -5e-324, 2.2250738585072014e-308, 1.7976931348623157e308,
        9007199254740993.0, 0.30000000000000004, 1.0 / 3.0, -2.5e-10, 4.35, 0.1 + 0.2, f64::NAN, f64::INFINITY,
        f64::NEG_INFINITY, f64::EPSILON, 1e16, 12345678901234567890.0, 0.000123, 100e-9,
    ];
    for v in values {
        println!("{}", b.format(v));
    }
    let mut c = b.clone();
    println!("{} {}", c.format(42.0), b.format(-7.25));
    let mut d: ryu_js::Buffer = Default::default();
    println!("{}", d.format(1e300));
}
