// expect: L0112
// rustc E0277: `&mut u8` is not Clone, so `VecDeque<&mut u8>` is not either
// (its `impl<T: Clone> Clone` does not apply).
use std::collections::VecDeque;
#[derive(Clone)]
struct S<'a>(VecDeque<&'a mut u8>);
fn main() {}
