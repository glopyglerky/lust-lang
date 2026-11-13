// expect: L0306
// rustc E0716: a block whose tail is such an `if` passes the reborrowed arm on.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    D(n)
}
fn len(d: &D) -> usize {
    d.0.len()
}
fn main() {
    let c = len(&D("c")) == 1;
    { if c { &mk("t1") } else { &mk("t2") } };
    println!("after");
}
