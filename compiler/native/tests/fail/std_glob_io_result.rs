// expect: L0110
// rustc E0107: `std::io::*` brings io::Result, which takes one argument.
use std::io::*;
fn f() -> Result<i32, String> { Ok(1) }
fn main() { println!("{:?}", f()); }
