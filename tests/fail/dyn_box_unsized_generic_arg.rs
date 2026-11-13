// expect: L0110
// rustc E0277: a generic fn's `T` cannot be `dyn T` (an unsized local).
fn id<T>(t: T) -> T { t }
trait T { fn v(&self) -> u8; }
struct A;
impl T for A { fn v(&self) -> u8 { 1 } }
fn main() { let b: Box<dyn T> = Box::new(A); let c = id(*b); println!("{}", c.v()); }
