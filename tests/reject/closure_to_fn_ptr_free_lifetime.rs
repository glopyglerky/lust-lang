// expect: L0203
// Valid Rust: a closure coerced to a fn pointer type that names a
// lifetime of the fn (W-BORROW3 cut).
fn f<'a>(_g: fn(&'a u8), x: &u8) -> &u8 { x }
fn main() { println!("{}", f(|_| {}, &1)); }
