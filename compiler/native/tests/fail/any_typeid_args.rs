// expect: L0105
// rustc E0061: `TypeId::of` takes no arguments (W-CHECK-G3).
use std::any::TypeId;
fn main() { println!("{}", TypeId::of::<u8>(5) == TypeId::of::<u8>()); }
