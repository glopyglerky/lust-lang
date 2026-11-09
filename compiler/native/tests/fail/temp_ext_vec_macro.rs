// expect: L0306
// rustc E0716: `vec!` elements are call arguments, not extended.
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
    let x = vec![&mk("t")];
    println!("{}", x[0].0);
}
