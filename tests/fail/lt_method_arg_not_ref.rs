// expect: L0110
// rustc E0308: lt takes &Rhs.
use std::cmp::Ordering;
struct P(u8);
impl PartialEq for P { fn eq(&self, o: &P) -> bool { self.0 == o.0 } }
impl PartialOrd for P { fn partial_cmp(&self, o: &P) -> Option<Ordering> { self.0.partial_cmp(&o.0) } }
fn main() { println!("{}", P(1).lt(P(2))); }
