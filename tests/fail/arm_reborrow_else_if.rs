// expect: L0306
// rustc E0716: an `else if` chain is reborrowed as a whole, so its first arm counts too.
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
    let n = len(&D("c"));
    if n == 0 { &mk("t1") } else if n == 1 { &mk("t2") } else { &mk("t3") };
    println!("after");
}
