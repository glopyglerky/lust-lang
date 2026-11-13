// expect: L0104
// rustc E0599: `dyn Any + Sync` has no `is` (std's impls are for Any, Any + Send, Any + Send + Sync) (W-CHECK-G3).
use std::any::Any;
fn main() { let x = 1; let a: &(dyn Any + Sync) = &x; println!("{}", a.is::<i32>()); }
