// expect: L0110
// rustc E0277: by-value `self` in an impl for an unsized type.
trait Bv { fn b(self) -> u8; }
impl Bv for str { fn b(self) -> u8 { 2 } }
fn main() {}
