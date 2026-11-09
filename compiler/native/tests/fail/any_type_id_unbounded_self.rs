// expect: L0612
// rustc E0310: `x.type_id()` on `&T` needs `T: 'static` (W-CHECK-G3).
use std::any::Any;
fn f<T>(x: &T) -> bool { x.type_id() == std::any::TypeId::of::<u8>() }
fn main() { println!("{}", f(&1u8)); }
