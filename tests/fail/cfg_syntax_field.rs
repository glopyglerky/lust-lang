// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
struct S { #[cfg(any())] a: , b: i32 }
fn main() { println!("ok"); }
