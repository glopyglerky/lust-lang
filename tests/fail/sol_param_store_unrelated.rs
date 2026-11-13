// expect: L0733
// Sol review probe `param_store_unrelated`; rustc: lifetime may not live long enough
fn bad<'a, 'b>(slot: &mut &'a i32, x: &'b i32) { *slot = x; }
fn main() {}
