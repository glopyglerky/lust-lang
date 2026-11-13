// expect: L0612
// rustc E0310: `TypeId::of::<T>()` needs `T: 'static` (W-CHECK-G3).
use std::any::TypeId;
fn f<T>() -> TypeId { TypeId::of::<T>() }
fn main() { println!("{}", f::<u8>() == TypeId::of::<u8>()); }
