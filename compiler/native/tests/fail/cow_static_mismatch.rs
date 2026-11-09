// expect: L0733
// rustc rejects: a borrowed Cow cannot be 'static (rustc lifetime error).
use std::borrow::Cow;
fn f<'a>(s: &'a str) -> Cow<'static, str> { Cow::Borrowed(s) }
fn main() { println!("{}", f("a")); }
