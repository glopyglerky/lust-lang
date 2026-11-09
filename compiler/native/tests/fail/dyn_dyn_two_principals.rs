// expect: L0110
// ARCH 2.8, rustc E0225: a dyn type has one principal trait plus auto traits.
trait A { fn a(&self); }
trait B { fn b(&self); }
fn f(_x: &(dyn A + B)) {}
fn main() {}
