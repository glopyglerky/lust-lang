// expect: L0203
// rustc E0581: the output names a binder lifetime that no input names.
fn ap(f: for<'a> fn() -> &'a u8) -> u8 { *f() }
fn main() { let _ = ap; }
