// expect: L0612
// ARCH 2.8, rustc E0277: `&&S` to `&dyn Tr` needs `&S: Tr`; rustc tries no deref coercion.
trait Tr { fn m(&self) -> u8; }
struct S;
impl Tr for S { fn m(&self) -> u8 { 1 } }
fn main() { let s = S; let r = &s; let d: &dyn Tr = &r; println!("{}", d.m()); }
