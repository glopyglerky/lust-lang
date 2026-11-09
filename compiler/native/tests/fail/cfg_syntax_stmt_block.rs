// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
fn main() { #[cfg(any())] { let x = 1 + ; } println!("ok"); }
