// expect: L0112
// rustc E0204: Range and RangeFrom are not Copy.
#[derive(Clone, Copy)]
struct C { a: std::ops::RangeFrom<u8> }
fn main() { let c = C { a: 1.. }; println!("{}", c.a.start); }
