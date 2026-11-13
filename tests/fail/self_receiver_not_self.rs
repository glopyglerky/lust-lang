// expect: L0110
// rustc E0307: in a trait, `Rc<S>` does not deref to the trait's Self.
use std::rc::Rc;
struct S { x: i32 }
trait G { fn get(self: Rc<S>) -> i32; }
impl G for S { fn get(self: Rc<S>) -> i32 { self.x } }
fn main() { println!("{}", Rc::new(S { x: 1 }).get()); }
