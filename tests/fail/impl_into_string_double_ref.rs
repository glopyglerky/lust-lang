// expect: L0612
// rustc E0277: std has `From<&str> for String`, not `From<&&str>`.
fn take(s: impl Into<String>) -> String { s.into() }
fn main() { println!("{}", take(&&"x")); }
