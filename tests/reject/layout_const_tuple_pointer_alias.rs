// expect: L0110
// Valid Rust: tuple DST pointer metadata is not implemented.
type Pointer = *const (u8, str);
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 16] }
fn main() {}
