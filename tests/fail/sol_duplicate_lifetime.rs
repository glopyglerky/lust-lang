// expect: L0106
// Sol review probe `duplicate_lifetime`; rustc: E0403 the name `'a` is already used for a generic parameter in this item's generic parameters
fn bad<'a, 'a>(x: &'a i32) {}
fn main() {}
