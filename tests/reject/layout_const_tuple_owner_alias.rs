// expect: L0110
// Valid Rust: owning tuple DST metadata is not implemented.
type Pointer = Box<(u8, str)>;
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 16] }
fn main() {}
