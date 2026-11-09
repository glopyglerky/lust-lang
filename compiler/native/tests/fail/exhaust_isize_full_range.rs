// expect: L0115
fn f(x: isize) -> u8 {
    match x {
        isize::MIN..=isize::MAX => 0,
    }
}
fn main() {
    println!("{}", f(7));
}
