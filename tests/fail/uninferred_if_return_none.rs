// expect: L0610
// rustc E0282: a `return` branch does not make `Option<_>` diverge.
fn main() {
    let _ = if true { return } else { None };
    println!("x");
}
