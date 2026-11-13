// expect: L0306
// rustc E0716: a method receiver is not extending, even a constructor call.
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
    let x = Some(&mk("t")).unwrap();
    println!("{}", x.0);
}
