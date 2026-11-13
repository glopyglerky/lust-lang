// expect: L0306
// rustc E0716: a field of a tuple expression is not extending; the tuple drops at the end of the `let`.
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
    let x = (&mk("t"),).0;
    println!("{}", x.0);
}
