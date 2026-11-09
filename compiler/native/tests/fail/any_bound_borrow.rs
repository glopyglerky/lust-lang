// expect: L0306
// rustc E0597: a `T: Any` argument must be 'static, so a local borrow fails (W-CHECK-G3).
use std::any::Any;
fn g<T: Any>(t: T) -> bool { t.type_id() == std::any::TypeId::of::<u8>() }
fn main() { let s = String::new(); println!("{}", g(&s)); }
