// expect: L0110
// Valid Rust: general builtin ADTs await the later layout evaluator.
fn bytes() -> [u8; core::mem::size_of::<core::mem::MaybeUninit<u32>>()] { [0; 4] }
fn main() {}
