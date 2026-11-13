// expect: L0115
fn f(x: usize) -> u8 {
    match x {
        0..=usize::MAX => 0,
    }
}
fn main() {
    println!("{}", f(7));
}
