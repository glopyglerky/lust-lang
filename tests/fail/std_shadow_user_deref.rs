// expect: L0643
// rustc E0277: through `W: Deref<Target = Vec<u8>>`, rustc's probe at the
// `Vec<u8>` step takes Vec's by-value `leak` before the user trait's
// `&self` method, and `&mut [u8]` is not Display.
use std::ops::Deref;
struct W(Vec<u8>);
impl Deref for W {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}
trait T {
    fn leak(&self) -> u8;
}
impl T for Vec<u8> {
    fn leak(&self) -> u8 {
        3
    }
}
fn main() {
    let w = W(vec![1]);
    println!("{}", w.leak());
}
