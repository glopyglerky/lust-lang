// expect: L0306
// rustc E0716: the `break` temporary drops before `let _` mentions the value.
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
    let _ = loop {
        break &mk("t");
    };
    println!("in");
}
