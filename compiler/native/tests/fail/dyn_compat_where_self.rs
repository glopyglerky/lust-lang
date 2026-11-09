// expect: L0701
// ARCH 2.8, rustc E0038: a method `where` clause on `Self` (other than Sized).
trait Other {}
trait T { fn f(&self) where Self: Other; }
fn f(_x: &dyn T) {}
fn main() {}
