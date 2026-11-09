// expect: L0610
// rustc E0282: an expression statement whose type nothing fixes.
fn main() {
    Vec::new();
    println!("x");
}
