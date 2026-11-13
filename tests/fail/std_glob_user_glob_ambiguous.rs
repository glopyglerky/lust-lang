// expect: L0505
// rustc E0659: `min` is ambiguous (the user glob and std::cmp::*).
// lust rejects a user-glob name beside a std glob: it cannot list a std module's names.
mod m { pub fn min(a: i32, b: i32) -> i32 { a * 100 + b } }
use m::*;
use std::cmp::*;
fn main() { println!("{}", min(1, 2)); }
