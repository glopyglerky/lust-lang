// expect: L0110
// rustc E0277: `*b` for `b: Box<dyn T>` has no static size, so it cannot be a local.
trait T { fn v(&self) -> u8; }
struct A;
impl T for A { fn v(&self) -> u8 { 1 } }
fn main() { let b: Box<dyn T> = Box::new(A); let x = *b; println!("{}", x.v()); }
