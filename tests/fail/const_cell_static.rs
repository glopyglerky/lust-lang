// expect: L0612
// rustc E0277: `Cell<u8>` is not Sync, so it cannot be a static.
use std::cell::Cell;
static S: Cell<u8> = Cell::new(1);
fn main() { println!("{}", S.get()); }
