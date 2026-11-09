// capability: rejected: `extend` with items of type `&T` into a VecDeque, BTreeSet, BTreeMap, IndexMap, or IndexSet whose element a later use fixes to `T` (their `Extend<&T>`)
// expect: L9001

use std::collections::VecDeque;

fn main() {
    let mut d = VecDeque::new();
    let v = vec![1u32];
    d.extend(v.iter());
    let x: u32 = d[0];
    println!("{x}");
}
