// expect: L0306
// rustc rejects: the matched &String borrows the Cow, which dies first.
use std::borrow::Cow;
fn main() { let k: &str; { let c: Cow<str> = Cow::Owned("x".to_string()); k = match &c { Cow::Owned(s) => s.as_str(), Cow::Borrowed(s) => s }; } println!("{}", k); }
