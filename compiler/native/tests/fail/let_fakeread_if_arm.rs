// expect: L0306
// rustc E0597: an `if` arm block local dies before the binding is used.
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
    let c = mk("c").0.len() == 1;
    let _x = if c {
        let y = D("y");
        &y
    } else {
        &D("lit")
    };
    println!("in");
}
