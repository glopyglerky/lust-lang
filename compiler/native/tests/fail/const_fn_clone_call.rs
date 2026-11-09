// expect: L0631
// rustc E0658: Clone::clone is only conditionally const in 1.99.
const fn h(x: i32) -> i32 { x.clone() }
fn main() { println!("{}", h(3)); }
