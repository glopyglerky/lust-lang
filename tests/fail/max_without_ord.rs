// expect: L0104
// rustc E0599: max needs Ord; only PartialOrd is written.
use std::cmp::Ordering;
#[derive(Debug)]
struct P(u8);
impl PartialEq for P { fn eq(&self, o: &P) -> bool { self.0 == o.0 } }
impl PartialOrd for P { fn partial_cmp(&self, o: &P) -> Option<Ordering> { self.0.partial_cmp(&o.0) } }
fn main() { println!("{:?}", P(1).max(P(2))); }
