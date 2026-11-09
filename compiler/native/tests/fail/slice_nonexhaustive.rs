// expect: L0115
fn f(s: &[u8]) -> u32 {
    match s {
        [] => 0,
        [_] => 1,
        [_, _] => 2,
    }
}
fn main() {
    println!("{}", f(&[]));
}
