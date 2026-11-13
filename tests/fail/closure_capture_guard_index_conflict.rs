// expect: L0303
// Indexing a captured guard mutably borrows the guard (`deref_mut`).
use std::cell::RefCell;
fn main() { let c = RefCell::new(vec![1u32, 2]); let mut g = c.borrow_mut(); let mut f = || g[0] = 7; let n = g.len(); f(); println!("{}", n); }
