// expect: L0612
// rustc E0277: an associated type has an implicit Sized bound.
trait Tr { type A; }
impl Tr for u8 { type A = str; }
fn main() { println!("x"); }
