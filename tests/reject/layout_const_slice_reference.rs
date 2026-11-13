// expect: L0110
// Valid Rust: unsized leaves are outside the early signature grammar.
fn bytes() -> [u8; core::mem::size_of::<&'static [u8]>()] { [0; 16] }
fn main() {}
