// expect: L0306
// rustc rejects: the Cow borrows `s`, which dies first (rustc E0597).
use std::borrow::Cow;
fn main() { let c; { let s = String::from("x"); c = Cow::Borrowed(s.as_str()); } println!("{}", c); }
