// expect: L0110
// rustc E0600: a negative value for a usize const param.
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
    println!("{}", len::<{ -1 }>());
}
