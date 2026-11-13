// expect: L0612
// ARCH 2.8, rustc E0277: S does not implement the dyn principal.
trait Tr { fn m(&self) -> u8; }
struct S;
fn main() { let s = S; let _d: &dyn Tr = &s; }
