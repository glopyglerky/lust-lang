// expect: L0110
type Pointer = *const (u8, str);
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 8] }
fn main() {}
