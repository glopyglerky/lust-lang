// expect: L0612
// rustc E0277: `Vec<u8>` is not Copy, so it cannot bind a Copy assoc type.
trait Tr { type A: Copy; }
impl Tr for u8 { type A = Vec<u8>; }
fn main() { println!("x"); }
