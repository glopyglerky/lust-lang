// expect: L0701
// ARCH 2.8, rustc E0038: `Self` in a parameter, even behind `&`.
trait Bad { fn other(&self, value: &Self); }
fn use_object(_x: &dyn Bad) {}
fn main() {}
