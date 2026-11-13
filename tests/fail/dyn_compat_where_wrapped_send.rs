// expect: L0701
// ARCH 2.8, rustc E0038: only `where Self: Send` itself keeps the trait dyn compatible; an auto-trait
// bound on a type that wraps `Self` references `Self` in the `where` clause.
trait T { fn f(&self) -> u8; fn h(&self) -> u8 where (Self,): Send; }
fn f(_x: &dyn T) {}
fn main() {}
