// expect: L0505
// rustc E0659: `min` is ambiguous between the two globs of one block.
mod m { pub fn min(a: i32, b: i32) -> i32 { a * 100 + b } }
fn main() {
    use m::*;
    use std::cmp::*;
    println!("{}", min(1, 2));
}
