// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
fn f(a: i32) {}
fn main() { f(#[cfg(any())] (1 +)); println!("ok"); }
