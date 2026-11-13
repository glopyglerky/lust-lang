// expect: L0631
// rustc E0658: range indexing is only conditionally const in 1.99.
const fn h(s: &[i32]) -> &[i32] { &s[0..1] }
fn main() { println!("{:?}", h(&[1, 2])); }
