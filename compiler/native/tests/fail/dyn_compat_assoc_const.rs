// expect: L0701
// ARCH 2.8, rustc E0038: an associated const makes a trait not dyn compatible.
trait T { const C: u8; }
fn f(_x: &dyn T) {}
fn main() {}
