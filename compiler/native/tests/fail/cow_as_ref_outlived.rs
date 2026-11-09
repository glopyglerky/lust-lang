// expect: L0306
// rustc rejects: as_ref borrows the Cow, which dies first.
use std::borrow::Cow;
fn main() { let r: &str; { let c: Cow<str> = Cow::Owned("x".to_string()); r = c.as_ref(); } println!("{}", r); }
