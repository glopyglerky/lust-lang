// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
fn main() { match 1 { #[cfg(any())] 1 => 1 +, _ => println!("ok"), } }
