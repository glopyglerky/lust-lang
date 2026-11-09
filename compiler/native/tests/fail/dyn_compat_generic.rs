// expect: L0701
// ARCH 2.8, rustc E0038: a method with its own type params has no single vtable entry.
trait T { fn g<U>(&self, x: U); }
fn f(_x: &dyn T) {}
fn main() {}
