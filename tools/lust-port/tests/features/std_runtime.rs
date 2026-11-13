// feature: std.runtime
// form: VecDeque and BTreeMap with ordered iteration
use std::collections::{BTreeMap, VecDeque};
fn main() {
    let mut q = VecDeque::new();
    q.push_back(2);
    q.push_front(1);
    let mut m = BTreeMap::new();
    m.insert(3, "c");
    m.insert(1, "a");
    let keys: Vec<i32> = m.keys().copied().collect();
    println!("{:?} {:?}", q.pop_front().unwrap_or(0), keys);
}
