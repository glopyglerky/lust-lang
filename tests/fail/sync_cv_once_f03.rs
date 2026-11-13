// expect: L0612
use std::sync::OnceLock;
use std::cell::Cell;
static C: OnceLock<Cell<u8>> = OnceLock::new();
fn main() { println!("{:?}", C.get().is_some()); }
