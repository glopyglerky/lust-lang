// expect: L0311
// rustc E0505: a temporary impl Trait value is dropped at the end of the statement, after the move (W-CHECK-G3).
use std::fmt::Display;
struct W<'a>(&'a String);
impl Display for W<'_> { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.0) } }
fn make(x: &String) -> impl Display + '_ { W(x) }
fn main() { let s = String::from("a"); let _ = (make(&s), drop(s)); }
