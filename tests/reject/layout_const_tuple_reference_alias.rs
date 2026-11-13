// expect: L0110
// Valid Rust: tuple DST pointer metadata is not implemented.
type Bytes = [u8; core::mem::size_of::<&(u8, str)>()];
fn force_alias_evaluation() -> Bytes { [0; 16] }
fn main() {}
