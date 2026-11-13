// expect: L0110
// rustc E0308: then_with needs a closure returning Ordering.
use std::cmp::Ordering;
fn main() { println!("{:?}", Ordering::Equal.then_with(|| 5)); }
