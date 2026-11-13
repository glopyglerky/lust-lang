// expect: L0612
// rustc: `NonZero<T>` requires an integer `T` (sealed primitive).
use std::num::NonZero;
fn f(x: NonZero<String>) {}
fn main() {}
