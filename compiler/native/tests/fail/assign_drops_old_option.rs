// expect: L0303
// rustc E0502: `h = None` drops the old `Some(G(&mut x))` in place.
struct G<'a>(&'a mut i32);
impl Drop for G<'_> {
    fn drop(&mut self) {
        println!("drop G {}", self.0);
    }
}
fn main() {
    let mut x = 1;
    let mut h = Some(G(&mut x));
    println!("x {}", x);
    h = None;
}
