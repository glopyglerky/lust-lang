// expect: L0303
// `wr(&mut x)` with `x: RefMut<P>` in a closure is `deref_mut(&mut x)`:
// a mutable borrow of the guard that conflicts with a read.
use std::cell::RefCell;
struct P { n: u32 }
fn wr(p: &mut P) -> u32 { p.n += 1; p.n }
fn main() { let c = RefCell::new(P { n: 1 }); let mut x = c.borrow_mut(); let mut f = || wr(&mut x); println!("{}", x.n); println!("{}", f()); }
