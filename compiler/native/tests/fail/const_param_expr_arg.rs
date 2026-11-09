// expect: L0110
// rustc: generic parameters may not be used in const operations.
fn small<const N: u8>() -> u8 {
    N
}
fn flag<const F: bool>() -> bool {
    F
}
fn len<const N: usize>() -> usize {
    N
}
fn plus<const N: usize>() -> usize {
    len::<{ N + 1 }>()
}
fn main() {
    println!("{}", plus::<2>());
}
