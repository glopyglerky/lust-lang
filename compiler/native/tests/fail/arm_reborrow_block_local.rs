// expect: L0306
// rustc E0597: the reborrow of the `else` arm happens after its block local dies.
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
    if c { &D("lit") } else { let y = D("y"); &y };
    println!("after");
}
