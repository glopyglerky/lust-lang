// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
enum E { #[cfg(any())] A = 1 +, B }
fn main() { println!("ok"); }
