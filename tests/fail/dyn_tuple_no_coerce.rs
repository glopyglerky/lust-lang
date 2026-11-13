// expect: L0110
// rustc E0308: a tuple value (not a tuple literal) does not coerce element-wise.
trait Tr { fn f(&self) -> u32; }
struct A(u32);
impl Tr for A { fn f(&self) -> u32 { self.0 } }
fn main() { let a = A(1); let t = (&a, 2u32); let u: (&dyn Tr, u32) = t; println!("{}", u.0.f()); }
