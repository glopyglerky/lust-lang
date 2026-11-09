// expect: L0306
// rustc E0716: a method receiver temporary is not extended; it drops at the end of the `let`.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    D(n)
}
impl D {
    fn me(&self) -> &D {
        self
    }
}
fn main() {
    let x = &mk("t").me();
    println!("{}", x.0);
}
