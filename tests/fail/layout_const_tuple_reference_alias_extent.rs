// expect: L0110
type Bytes = [u8; core::mem::size_of::<&(u8, str)>()];
fn force_alias_evaluation() -> Bytes { [0; 8] }
fn main() {}
