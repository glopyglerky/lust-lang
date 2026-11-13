// expect: L0615
// rustc E0119: core's `impl<T: Display + ?Sized> ToString for T` already covers L.
use std::fmt;
struct L;
impl fmt::Display for L { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "L") } }
impl ToString for L { fn to_string(&self) -> String { "mine".into() } }
fn main() { println!("{}", L.to_string()); }
