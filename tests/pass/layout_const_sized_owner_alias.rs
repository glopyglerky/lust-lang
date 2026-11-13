type Pointer = Box<(u8, u32)>;
type Bytes = [u8; core::mem::size_of::<Pointer>()];
fn force_alias_evaluation() -> Bytes { [0; 8] }
fn main() { println!("{}", force_alias_evaluation().len()); }
