// A std collection of a type with no `Debug` builds when nothing formats
// it: the glue makes no instance of the `T: Debug` impl whose bound fails.
use std::cell::OnceCell;
use std::collections::{BTreeMap, VecDeque};
struct N(u8);
fn main() {
    let mut q: VecDeque<N> = VecDeque::new();
    q.push_back(N(1));
    let mut m: BTreeMap<u8, N> = BTreeMap::new();
    m.insert(1, N(2));
    let c: OnceCell<N> = OnceCell::new();
    let _ = c.set(N(3));
    let a = match m.get(&1) {
        Some(n) => n.0,
        None => 0,
    };
    let b = match c.get() {
        Some(n) => n.0,
        None => 0,
    };
    println!("{} {} {}", q.len(), a, b);
}
