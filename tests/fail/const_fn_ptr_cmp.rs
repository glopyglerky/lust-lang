// expect: L0630
// rustc: fn pointers cannot be compared in const eval.
fn one() -> u8 {
    1
}
const F: fn() -> u8 = one;
const X: bool = F == F;
fn main() {
    println!("{}", X);
}
