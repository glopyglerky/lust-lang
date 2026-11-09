// expect: L0115
fn f(s: &[u8]) -> u8 {
    match s {
        [] => 0,
    }
}
fn main() {
    println!("{}", f(&[]));
}
