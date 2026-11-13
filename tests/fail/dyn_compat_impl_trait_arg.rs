// expect: L0701
// ARCH 2.8, rustc E0038: `impl Trait` in an argument is a generic method.
trait T { fn f(&self, x: impl Copy); }
fn f(_x: &dyn T) {}
fn main() {}
