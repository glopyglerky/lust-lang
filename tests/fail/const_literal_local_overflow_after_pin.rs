// expect: L0630
// rustc E0080: `h` is a u8 (from `h + a`), so `200 + 200` overflows; the
// evaluator must not run it wide and narrow the result.
const fn f(a: u8) -> u8 {
    let mut h = 0;
    h = h + a;
    h = h + a;
    h - a
}
const X: u8 = f(200);
fn main() {
    println!("{X}");
}
