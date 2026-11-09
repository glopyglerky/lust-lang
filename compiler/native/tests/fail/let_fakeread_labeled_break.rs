// expect: L0306
// rustc E0716: the labeled `break` temporary drops before the binding is used.
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
    let _x = 'a: {
        break 'a &mk("t");
    };
    println!("in");
}
