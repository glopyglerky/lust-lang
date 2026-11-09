// expect: L0001
// rustc: `limit` must be a non-negative integer.
#![recursion_limit = "1_000"]
macro_rules! one { () => { 1 } }
fn main() { println!("{}", one!()); }
