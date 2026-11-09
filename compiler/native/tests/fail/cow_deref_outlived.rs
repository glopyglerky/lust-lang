// expect: L0306
// rustc E0597: the &str from the coercion borrows the Cow, which dies first.
use std::borrow::Cow;
fn main() { let r: &str; { let c: Cow<str> = Cow::Owned("x".to_string()); r = &c; } println!("{}", r); }
