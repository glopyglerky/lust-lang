// expect: L0705
// Valid Rust (T is inferred as &'static str); lust cannot prove the region of a reference behind `downcast_mut`, so it rejects (W-CHECK-G3).
use std::any::Any;
fn main() { let x = 3; let a: &mut dyn Any = &mut 5; let m: Option<&mut &str> = a.downcast_mut(); let _ = (x, m); }
