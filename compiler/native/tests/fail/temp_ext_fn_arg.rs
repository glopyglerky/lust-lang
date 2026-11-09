// expect: L0306
// rustc E0716: a call argument is not an extending expression.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    D(n)
}
fn id<T>(x: T) -> T {
    x
}
fn main() {
    let x = id(&mk("t"));
    println!("{}", x.0);
}
