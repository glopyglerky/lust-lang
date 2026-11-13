// expect: L0110
// rustc E0277: an unsized value cannot be passed by value.
trait T { fn v(&self) -> u8; }
struct A;
impl T for A { fn v(&self) -> u8 { 1 } }
fn main() { let b: Box<dyn T> = Box::new(A); drop(*b); }
