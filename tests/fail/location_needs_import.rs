// expect: L0101
// rustc E0433: `Location` needs `use std::panic::Location` (W-CHECK-G3).
fn main() { println!("{}", Location::caller().line()); }
