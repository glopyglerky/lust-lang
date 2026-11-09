// expect: L0305
// rustc E0507: `*w` through a user `Deref` is a place behind a reference.
use std::ops::Deref;
struct W { s: String }
impl Deref for W { type Target = String; fn deref(&self) -> &String { &self.s } }
fn main() { let w = W { s: String::from("x") }; let s: String = *w; println!("{}", s); }
