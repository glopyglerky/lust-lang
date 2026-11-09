// expect: L0612
// rustc E0277: `T: Add<u64, Output = u64>` makes the argument u64, which has no `Neg`.
use std::ops::Add;
fn add<T: Add<u64, Output = u64>>(t: T) -> u64 {
    t + 1
}
fn main() {
    println!("{}", add(-5));
}
