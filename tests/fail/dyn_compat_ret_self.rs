// expect: L0701
// ARCH 2.8, rustc E0038: `Self` as a return type.
trait Bad { fn copy(&self) -> Self; }
fn f(_x: &dyn Bad) {}
fn main() {}
