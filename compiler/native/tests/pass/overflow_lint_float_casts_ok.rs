// Known float casts that do not overflow: saturation, NaN to 0, f32
// rounding, and a parenthesized half-open range in matches!.
#[derive(Clone, Copy)]
#[repr(i8)]
enum E {
    A = -1,
    B = 7,
}
fn main() {
    let a = 300.0f64 as u8;
    println!("{}", a - 1);
    let n = f64::NAN as u8;
    println!("{}", n + 1);
    let r = 3.9f32 as i8 * 30;
    println!("{}", r);
    let w = 254.6f32 as u8;
    println!("{}", w + 1);
    let big = 1e20 as i64;
    println!("{}", big - 1);
    let m = (300.0f32 as u8) as u16 * 200;
    println!("{}", m);
    println!("{} {}", E::A as u8 - 1, E::B as i8 + 1);
    let b = Box::new([1u8, 2]);
    println!("{}", b[1]);
    println!("{}", matches!(-3, (..=0)));
    println!("{}", -(2.5f32) as i32 + 1);
    let q = (1.5f64 * 100.0 - 0.5) as u8;
    println!("{}", q + 106);
}
