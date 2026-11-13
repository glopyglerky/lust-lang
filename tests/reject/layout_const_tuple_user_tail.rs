// expect: L0110
// Valid Rust: a user Adt tuple tail needs a Sized proof unavailable here.
struct Tail(u8);
type Pointer = *const (u8, Tail);
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 8] }
fn main() {}
