// expect: L0306
// rustc E0716: a match scrutinee is not extending.
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
    let x = match &mk("t") {
        r => r,
    };
    println!("{}", x.0);
}
