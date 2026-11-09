// feature: ext.ryu_js
// form: ryu_js::Buffer::format, the JS Number to string of jsnum.rs Display
// cargo-dep: ryu-js = "=1.0.2"
fn main() {
    let mut b = ryu_js::Buffer::new();
    for x in [0.1f64, 1e21, 1.5e-7, -0.0, 123456789.0, f64::NAN, f64::INFINITY] {
        println!("{}", b.format(x));
    }
}
