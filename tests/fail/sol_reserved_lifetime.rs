// expect: L0203
// Sol review probe `reserved_lifetime`; rustc: E0262 invalid lifetime parameter name: `'static`
fn bad<'static>(x: &'static i32) {}
fn main() {}
