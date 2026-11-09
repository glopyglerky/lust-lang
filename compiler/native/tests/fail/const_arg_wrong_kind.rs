// expect: L0110
// rustc E0308: an integer for a bool const param.
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
    println!("{}", flag::<1>());
}
