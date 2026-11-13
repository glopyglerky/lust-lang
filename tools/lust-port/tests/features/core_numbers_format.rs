// feature: core.numbers_format
// form: u128 and f64 values, float literal, width precision and radix format specs
fn main() {
    let big: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455;
    let f: f64 = 1.5;
    println!("{} {:.2} {:08x} {:>6.1}", big % 100, f * 2.0, 255, 3.25f32);
}
