// expect: L0306
// rustc E0716: match arms after the first one that does not diverge are reborrowed.
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
    match n {
        0 => return,
        1 => { &mk("a") }
        _ => { &mk("b") }
    };
    println!("after");
}
