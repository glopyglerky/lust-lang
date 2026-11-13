// expect: L0306
// rustc E0597: a `let` binding is used once its initializer is done (FakeRead), so a block local it borrows must still live.
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
    let _x = {
        let y = D("y");
        &y
    };
    println!("in");
}
