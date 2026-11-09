// expect: L0303
// rustc rejects: to_mut keeps the Cow mutably borrowed (rustc E0502).
use std::borrow::Cow;
fn main() { let mut c: Cow<str> = Cow::Borrowed("a"); let m = c.to_mut(); println!("{}", c); m.push('b'); }
