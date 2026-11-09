// expect: L0308
// rustc E0596: `&mut c` needs a mutable binding.
fn g(f: &mut dyn FnMut()) { f() }
fn main() { let mut n = 0; let c = || n += 1; g(&mut c); }
