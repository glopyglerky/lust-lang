// expect: L0110
// rustc E0308: the path written type is the Self of the item (was accepted).
use std::sync::{Arc, Weak};
fn main() { let a = Arc::new(1u8); let w = Arc::downgrade(&a); let _ = Weak::<u16>::upgrade(&w); }
