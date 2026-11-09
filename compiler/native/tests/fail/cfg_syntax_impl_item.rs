// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
struct S;
impl S { #[cfg(any())] fn f(&self) { 1 + } }
fn main() { println!("ok"); }
