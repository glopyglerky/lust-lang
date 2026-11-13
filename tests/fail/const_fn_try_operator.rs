// expect: L0631
// rustc E0658: `?` is not allowed in a const fn.
const fn h(x: Option<i32>) -> Option<i32> { let v = x?; Some(v + 1) }
fn main() { println!("{:?}", h(Some(3))); }
