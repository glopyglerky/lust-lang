// expect: L0110
// Valid Rust: this early grammar requires a recursively Sized pointee.
fn bytes() -> [u8; core::mem::size_of::<*const (u8, str)>()] { [0; 16] }
fn main() {}
