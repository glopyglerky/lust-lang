// expect: L0306
// rustc E0597: `let _ = value` mentions its temporary after the initializer.
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
    let _ = {
        let y = D("y");
        &y
    };
    println!("in");
}
