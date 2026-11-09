// ARCH 2.2 and 2.10: core's default bodies for a written impl that gives
// only the required methods: `!=` and `ne` through `eq`, `<` and `lt`
// (and kin) through `partial_cmp`, and Ord::max/min/clamp through
// PartialOrd::lt/le/gt on Self, in rustc's call order.
use std::cmp::Ordering;
#[derive(Debug)]
struct V(i32);
impl PartialEq for V {
    fn eq(&self, o: &V) -> bool {
        println!("eq {} {}", self.0, o.0);
        self.0 == o.0
    }
}
impl Eq for V {}
impl PartialOrd for V {
    fn partial_cmp(&self, o: &V) -> Option<Ordering> {
        println!("pcmp {} {}", self.0, o.0);
        self.0.partial_cmp(&o.0)
    }
}
impl Ord for V {
    fn cmp(&self, o: &V) -> Ordering {
        println!("cmp {} {}", self.0, o.0);
        self.0.cmp(&o.0)
    }
}
fn main() {
    println!("{} {}", V(1) != V(2), V(1).ne(&V(1)));
    println!("{} {} {}", V(1) < V(2), V(3).ge(&V(3)), V(2) > V(1));
    println!("{:?} {:?} {:?}", V(1).max(V(2)), V(1).min(V(2)), V(9).clamp(V(0), V(5)));
}
