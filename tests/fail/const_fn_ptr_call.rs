// expect: L0630
// rustc E0015: a fn pointer call is not allowed in const.
const fn ap(f: fn() -> u8) -> u8 {
    f()
}
fn one() -> u8 {
    1
}
const X: u8 = ap(one);
fn main() {
    println!("{}", X);
}
