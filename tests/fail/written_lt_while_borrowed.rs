// expect: L0303
// rustc E0502: < borrows both sides.
use std::cmp::Ordering;
struct P(Vec<u8>);
impl PartialEq for P { fn eq(&self, o: &P) -> bool { self.0 == o.0 } }
impl PartialOrd for P { fn partial_cmp(&self, o: &P) -> Option<Ordering> { self.0.partial_cmp(&o.0) } }
fn main() { let mut a = P(vec![1]); let r = &mut a; println!("{}", a < P(vec![2])); r.0.push(1); }
