// expect: L0112
// Valid Rust: a written PartialOrd that also writes lt is called through
// lt by core's tuple ordering; the cmp glue does not model that yet.
use std::cmp::Ordering;
#[derive(PartialEq)]
struct V(u8);
impl PartialOrd for V { fn partial_cmp(&self, o: &V) -> Option<Ordering> { println!("pcmp"); self.0.partial_cmp(&o.0) } fn lt(&self, o: &V) -> bool { println!("lt"); self.0 < o.0 } }
fn main() { println!("{}", (V(1), V(2)) < (V(1), V(3))); }
