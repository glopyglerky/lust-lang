// expect: L0703
// ARCH 2.8, rustc: a `where Self: Clone` method implies `Self: Sized`, so a trait object cannot call it.
trait T { fn f(&self) -> u8; fn g(&self) -> u8 where Self: Clone { 1 } }
#[derive(Clone)] struct S(u8);
impl T for S { fn f(&self) -> u8 { self.0 } }
fn main() { let s = S(3); let d: &dyn T = &s; println!("{}", d.g()); }
