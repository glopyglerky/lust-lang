// expect: L0115
fn f(x: u8) -> u8 {
    match x {
        0..=127 => 1,
        128..=254 => 2,
    }
}
fn main() {
    println!("{}", f(3));
}
