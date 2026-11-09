// expect: L0303
// rustc E0503: `h = G(&mut y)` drops the old `G`, which still holds
// `&mut x`, so the drop is a use of that borrow after `x += 1`.
struct G<'a>(&'a mut i32);
impl Drop for G<'_> {
    fn drop(&mut self) {
        println!("drop G {}", self.0);
    }
}
fn main() {
    let mut x = 1;
    let mut y = 2;
    let mut h = G(&mut x);
    x += 1;
    h = G(&mut y);
    println!("x {}", x);
}
