// expect: L0110
// Valid Rust: composite unsized metadata is outside the early grammar.
fn bytes() -> [u8; core::mem::size_of::<*const std::sync::Mutex<str>>()] { [0; 16] }
fn main() {}
