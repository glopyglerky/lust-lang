// expect: L0112
// ARCH 2.8, rustc E0277: `dyn Tr` has no Display impl.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
fn main() { let s = S; let d: &dyn Tr = &s; println!("{}", d); }
