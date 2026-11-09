// expect: L0733
// Sol review probe `shared_outer_mut_reborrow`; rustc: lifetime may not live long enough
fn bad<'a, 'b>(x: &'a &'b mut i32) -> &'b i32 { &**x }
fn main() {}
