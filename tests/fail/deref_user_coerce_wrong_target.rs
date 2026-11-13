// expect: L0110
// rustc E0308: the target `str` is not `u8`.
use std::ops::Deref;
struct Name(String);
impl Deref for Name { type Target = str; fn deref(&self) -> &str { &self.0 } }
fn takes(x: &u8) -> u8 { *x }
fn main() { let n = Name(String::new()); takes(&n); }
