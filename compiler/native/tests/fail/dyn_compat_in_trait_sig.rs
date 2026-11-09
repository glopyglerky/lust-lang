// expect: L0701
// ARCH 2.8, rustc E0038: the check runs at a dyn type in the trait's own method signature.
trait Bad { fn copy(&self) -> Self; fn g(&self, x: &dyn Bad); }
fn main() {}
