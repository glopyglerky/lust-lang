// expect: L0306
// rustc E0716: an array repeat operand is not an extending expression.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    D(n)
}
fn main() {
    let x = [&mk("t"); 1];
    println!("{}", x[0].0);
}
