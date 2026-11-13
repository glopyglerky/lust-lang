// expect: L0115
use std::sync::Mutex;
fn main() { let m = Mutex::new(5); match m.lock() { Ok(g) => println!("{}", *g), } }
