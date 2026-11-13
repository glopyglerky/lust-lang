// expect: L0308
// `Rc` has no `DerefMut`: no assignment through it in a closure.
use std::rc::Rc;
fn main() { let r = Rc::new([1u32, 2]); let i = 1; let mut f = || r[i] = 5; f(); }
