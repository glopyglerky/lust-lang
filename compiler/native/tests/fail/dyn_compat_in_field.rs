// expect: L0701
// ARCH 2.8, rustc E0038: the check runs at a dyn type in a field.
trait Bad { fn copy(&self) -> Self; }
struct H { d: &'static dyn Bad }
fn main() {}
