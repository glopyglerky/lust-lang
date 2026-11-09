// expect: L0705
// rustc E0310: a `&T` with T not 'static cannot become `&dyn Any` (W-CHECK-G3).
use std::any::Any;
fn f<T>(x: &T) -> &dyn Any { x }
fn main() { println!("{}", f(&1).is::<i32>()); }
