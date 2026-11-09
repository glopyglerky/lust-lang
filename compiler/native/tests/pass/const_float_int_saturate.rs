// Float-to-int `as` casts in const evaluation saturate at the
// target range (NaN becomes 0), in const items, statics, const
// fns, and array lengths alike.
const A: u8 = 256.0 as u8;
const B: i32 = -1e20 as i32;
const C: i64 = f64::MAX as i64;
static S: i8 = 300.5 as i8;
const fn f(x: f64) -> u16 {
    x as u16
}
const G: u16 = f(1e9);
const H: u32 = 4294967296.0f32 as u32;
const N: i32 = f64::NAN as i32;
const P: u8 = 300.0 as u8;
fn main() {
    println!("{} {} {} {} {} {}", A, B, C, S, G, H);
    println!("{}", N);
    println!("{}", [0u8; (300.0 as u8) as usize].len());
    match 255u8 {
        P => println!("match"),
        x => println!("no {}", x),
    }
}
