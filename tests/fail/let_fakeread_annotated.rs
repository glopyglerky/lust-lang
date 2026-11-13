// expect: L0306
// rustc E0597: an annotated binding is used once its initializer is done.
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
    let _x: &D = {
        let y = D("y");
        &y
    };
    println!("in");
}
