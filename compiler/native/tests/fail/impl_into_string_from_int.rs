// expect: L0612
// rustc E0277: `String: From<i32>` does not hold, so `i32: Into<String>` fails.
fn take(s: impl Into<String>) -> usize { s.into().len() }
fn main() { println!("{}", take(5)); }
