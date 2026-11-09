// expect: L0306
// rustc E0597: a match arm block local dies before the binding is used.
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
    let _x = match 1 {
        _ => {
            let y = D("y");
            &y
        }
    };
    println!("in");
}
