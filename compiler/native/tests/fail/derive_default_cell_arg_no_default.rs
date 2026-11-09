// expect: L0110
// rustc E0277: `No` is not Default, so `Cell<(No, u8)>` is not
// Default and the derive on `Memo` fails.
use std::cell::Cell;
struct No;
#[derive(Default)]
struct Memo(Cell<(No, u8)>);
fn main() { println!("x"); }
