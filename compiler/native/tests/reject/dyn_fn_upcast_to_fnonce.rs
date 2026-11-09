// expect: L0110
// Valid Rust (trait upcasting, 1.86): `Box<dyn Fn()>` to `Box<dyn FnOnce()>`. lust builds no super slots for the Fn family yet.
fn main() { let f: Box<dyn Fn()> = Box::new(|| ()); let g: Box<dyn FnOnce()> = f; g(); }
