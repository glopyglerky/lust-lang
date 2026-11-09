// expect: L0306
// rustc E0716: a `break` value is not an extending expression.
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
    let x = loop {
        break &mk("t");
    };
    println!("{}", x.0);
}
