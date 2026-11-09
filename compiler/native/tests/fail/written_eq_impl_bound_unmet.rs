// expect: L0612
// rustc E0369: the generic impl needs T: PartialOrd.
use std::cmp::Ordering;
struct W<T>(T);
impl<T: PartialOrd> PartialEq for W<T> { fn eq(&self, o: &W<T>) -> bool { self.0.partial_cmp(&o.0) == Some(Ordering::Equal) } }
struct N;
fn main() { println!("{}", W(N) == W(N)); }
