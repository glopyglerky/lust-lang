// An impl Trait value that borrows an argument, dropped before the argument moves (W-CHECK-G3).
use std::fmt::Display;
struct W<'a>(&'a String);
impl Display for W<'_> { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.0) } }
fn make(x: &String) -> impl Display + '_ { W(x) }
fn main() { let s = String::from("a"); { let d = make(&s); println!("{}", d); } drop(s); println!("ok"); }
