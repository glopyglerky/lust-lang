// `assert_eq!` with an open left type that a later use fixes, and with a
// literal on the left, both build in rustc; so do arithmetic operators
// whose right operand fixes its own type.
trait Pick {
    fn pick() -> Self;
}
impl Pick for i32 {
    fn pick() -> Self {
        10
    }
}
impl Pick for u8 {
    fn pick() -> Self {
        20
    }
}
fn make<T: Pick>() -> T {
    T::pick()
}
fn main() {
    let x = make();
    assert_eq!(x, 20);
    let y: u8 = x;
    let z = make();
    assert_eq!(10, z);
    let w: i32 = z;
    let n = 3u32;
    let s = 1u8 << n;
    let t = 2u64 * (y as u64) + 7;
    let u = 1.5f64 * 2.0 - f64::from(1u8);
    println!("{y} {w} {s} {t} {u}");
}

// A compound assignment whose open right side a later use fixes.
#[allow(dead_code)]
fn later() -> u8 {
    let mut x = 3u8;
    let r = make();
    x += r;
    let _f: u8 = r;
    x
}
