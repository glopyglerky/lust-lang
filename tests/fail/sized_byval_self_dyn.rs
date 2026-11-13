// expect: L0110
// rustc E0277: by-value `self` needs a sized Self type; `dyn Tr` is not.
trait Tr { fn m(&self) -> u8; }
impl dyn Tr { fn byv(self) -> u8 { 1 } }
fn main() {}
