// expect: L0110
// rustc E0308: `2i32` is no usize.
struct S { a: [u8; 2i32] }
fn main() { let s = S { a: [1, 2] }; println!("{}", s.a.len()); }
