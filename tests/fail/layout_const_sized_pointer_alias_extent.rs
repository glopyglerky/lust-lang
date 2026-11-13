// expect: L0110
type Pointer = *const (u8, u16);
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 16] }
fn main() {}
