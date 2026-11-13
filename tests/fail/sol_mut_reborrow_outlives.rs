// expect: L0733
// Sol review probe `mut_reborrow_outlives`; rustc: lifetime may not live long enough
fn bad<'a, 'b>(x: &'a mut &'b mut i32) -> &'b mut i32 { &mut **x }
fn main() {}
