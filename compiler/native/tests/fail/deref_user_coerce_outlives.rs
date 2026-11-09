// expect: L0306
// rustc E0597: the coerced `&str` borrows `n`, which ends first.
use std::ops::Deref;
struct Name(String);
impl Deref for Name { type Target = str; fn deref(&self) -> &str { &self.0 } }
fn main() { let s: &str; { let n = Name(String::from("x")); s = &n; } println!("{}", s); }
