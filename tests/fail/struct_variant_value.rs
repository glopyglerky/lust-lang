// expect: L0101
// rustc E0533: a struct variant is not a value.
enum E {
    S { a: u8 },
}
fn main() {
    let x = E::S;
    let _ = x;
}
