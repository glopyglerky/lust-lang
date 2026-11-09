// expect: L0110
enum Void {}
fn a(r: &Result<u8, Void>) -> u8 {
    let Ok(x) = *r;
    x
}
fn main() {
    println!("{}", a(&Ok(3)));
}
