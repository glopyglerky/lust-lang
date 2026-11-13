// expect: L0612
use std::cell::Cell;
static C: Option<Cell<u8>> = None;
fn main() { println!("{}", C.is_none()); }
