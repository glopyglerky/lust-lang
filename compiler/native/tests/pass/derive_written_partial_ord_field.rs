// A written PartialOrd (no Ord, no lt/le/gt/ge of its own) inside derived
// PartialOrd, tuples, Vec, and Option, and alone: one partial_cmp call per
// comparison, in rustc's order, NaN unordered.
use std::cmp::Ordering;
#[derive(Debug, PartialEq)]
struct V(f64);
impl PartialOrd for V { fn partial_cmp(&self, o: &V) -> Option<Ordering> { println!("pcmp {} {}", self.0, o.0); o.0.partial_cmp(&self.0) } }
#[derive(Debug, PartialEq, PartialOrd)]
struct S { v: V, k: u8 }
fn main() {
    println!("{}", S { v: V(1.0), k: 0 } < S { v: V(2.0), k: 0 });
    println!("{:?}", S { v: V(1.0), k: 1 }.partial_cmp(&S { v: V(1.0), k: 2 }));
    println!("{}", (V(1.0), 2) < (V(1.0), 3));
    println!("{}", vec![V(1.0), V(2.0)] <= vec![V(1.0), V(f64::NAN)]);
    println!("{:?}", Some(V(3.0)).partial_cmp(&Some(V(4.0))));
    println!("{} {}", V(1.0) < V(2.0), V(1.0) >= V(2.0));
}
