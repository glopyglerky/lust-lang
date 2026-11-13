// expect: L0110
// ARCH 2.8, rustc E0277: `*d` is unsized, so it cannot be a local's value.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
fn main() { let s = S; let d: &dyn Tr = &s; let _x = *d; }
