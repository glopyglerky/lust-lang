// expect: L0640
// Sorting compares with `PartialOrd::lt`; a written `lt` override
// (not core's default from `partial_cmp`) is one lust cannot call
// in its sort glue yet, so the sort rejects instead of ordering by
// `partial_cmp`.
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
struct K(i32);

impl PartialOrd for K {
    fn partial_cmp(&self, o: &K) -> Option<Ordering> {
        Some(self.cmp(o))
    }
    fn lt(&self, o: &K) -> bool {
        self.0 > o.0
    }
}

fn main() {
    let mut v = vec![K(1), K(3), K(2)];
    v.sort();
    println!("{:?}", v);
}
