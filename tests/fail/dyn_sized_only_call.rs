// expect: L0703
// ARCH 2.8, rustc: a `where Self: Sized` method cannot be invoked on a trait object.
trait Tr { fn copy(&self) -> Self where Self: Sized; fn h(&self) -> u8; }
struct S;
impl Tr for S { fn copy(&self) -> Self { S } fn h(&self) -> u8 { 7 } }
fn main() { let s = S; let d: &dyn Tr = &s; let _c = d.copy(); }
