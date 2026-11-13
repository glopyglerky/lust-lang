// expect: L0306
// rustc E0716: the `else` arm of a reference-typed `if` is reborrowed (LUB coercion) after its block drops its tail temporary, even when the value is unused.
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
    if c { &D("lit") } else { &mk("t2") };
    println!("after");
}
