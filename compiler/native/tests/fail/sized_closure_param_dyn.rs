// expect: L0110
// rustc E0277: a closure parameter is a value and must be sized.
trait Tr { fn m(&self) -> u8; }
fn main() {
    let g = |x: dyn Tr| 1u8;
    let _ = g;
}
