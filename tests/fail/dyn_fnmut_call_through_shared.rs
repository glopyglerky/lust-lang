// expect: L0308
// rustc E0596: calling a `dyn FnMut` needs `&mut` access.
fn g(f: &dyn FnMut()) { f() }
fn main() { g(&|| ()); }
