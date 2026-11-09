// expect: L0733
// Sol review probe `named_return_unrelated`; rustc: lifetime may not live long enough
fn bad<'a, 'b>(x: &'a i32) -> &'b i32 { x }
fn main() {}
