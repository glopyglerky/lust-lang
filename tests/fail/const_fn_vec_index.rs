// expect: L0631
// rustc E0658: Vec indexing goes through a conditionally const Index impl.
const fn h(v: Vec<i32>) -> i32 { v[0] }
fn main() { println!("{}", h(vec![7])); }
