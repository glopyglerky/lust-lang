// expect: L0110
// rustc E0161: a `dyn FnOnce` behind a reference cannot be called.
fn g(f: &dyn FnOnce()) { f() }
fn main() { g(&|| ()); }
