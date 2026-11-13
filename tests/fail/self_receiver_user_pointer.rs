// expect: L0110
// rustc E0658: a user smart pointer as the self type needs arbitrary_self_types.
use std::ops::Deref;
struct P<T>(T);
impl<T> Deref for P<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
struct S { x: i32 }
impl S { fn get(self: P<Self>) -> i32 { self.x } }
fn main() { let _ = P(S { x: 1 }); }
