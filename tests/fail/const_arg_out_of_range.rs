// expect: L0110
// rustc: literal out of range for `u8` (a const argument).
fn small<const N: u8>() -> u8 {
    N
}
fn flag<const F: bool>() -> bool {
    F
}
fn len<const N: usize>() -> usize {
    N
}
fn main() {
    println!("{}", small::<300>());
}
