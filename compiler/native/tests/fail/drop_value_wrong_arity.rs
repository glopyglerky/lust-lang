// expect: L0105
// rustc E0057: the prelude `drop` value takes one argument.
fn main() { let f = drop; f(1u8, 2u8); }
