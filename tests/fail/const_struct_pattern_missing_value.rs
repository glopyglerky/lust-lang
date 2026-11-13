// expect: L0115
// Consts cover only their own values: `Some(u8::MAX)` stays uncovered.
const A: Option<u8> = Some(0);
fn f(x: Option<u8>) -> u8 {
    match x {
        A => 1,
        Some(1..=254) => 2,
        None => 3,
    }
}
fn main() {
    println!("{}", f(None));
}
