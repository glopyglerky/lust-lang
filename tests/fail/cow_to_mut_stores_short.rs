// expect: L0306
// rustc rejects: to_mut stores a borrow of `s` in the owned Vec.
use std::borrow::Cow;
fn main() { let mut c: Cow<[&str]> = Cow::Owned(vec!["a"]); { let s = String::from("z"); c.to_mut().push(&s); } println!("{:?}", c); }
