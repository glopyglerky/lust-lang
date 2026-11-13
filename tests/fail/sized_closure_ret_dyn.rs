// expect: L0110
// rustc E0746/E0277: a closure return type must be sized.
trait Tr { fn m(&self) -> u8; }
fn main() {
    let g = || -> dyn Tr { loop {} };
    let _ = g;
}
