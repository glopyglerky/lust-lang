// expect: L0701
// ARCH 2.8, rustc E0038: an associated fn with no receiver.
trait T { fn new() -> u8; }
fn f(_x: &dyn T) {}
fn main() {}
