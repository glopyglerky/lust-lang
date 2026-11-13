// expect: L0610
// rustc E0282: a `_` pattern does not fix the element type.
fn main() {
    let _ = Vec::new();
    println!("x");
}
