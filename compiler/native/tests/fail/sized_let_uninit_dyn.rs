// expect: L0110
// rustc E0277: a local of an unsized type, initializer or not.
trait Tr { fn m(&self) -> u8; }
fn main() {
    let x: dyn Tr;
    let _ = &x;
}
