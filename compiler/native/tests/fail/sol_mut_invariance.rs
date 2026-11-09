// expect: L0733
// Sol review probe `mut_invariance`; rustc: lifetime may not live long enough
fn bad<'a, 'b: 'a>(x: &'a mut &'b i32) -> &'a mut &'a i32 { x }
fn main() {}
