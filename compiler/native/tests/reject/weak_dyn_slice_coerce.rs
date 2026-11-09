// expect: L0110
// Valid Rust: lust has `Weak<[u8]>` (from an `Rc<[u8]>`), but no unsizing
// coercion from `Weak<[u8; 3]>` (a dangling `Weak::new()` would need a fat
// dangling form).
use std::rc::{Rc, Weak};
fn main() {
    let s = Rc::new([1u8, 2, 3]);
    let w: Weak<[u8; 3]> = Rc::downgrade(&s);
    let d: Weak<[u8]> = w;
    println!("{}", d.upgrade().map_or(0, |x| x.len()));
}
