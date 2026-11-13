// panic: range start and end are equal and excluded in BTreeMap
use std::collections::BTreeMap;
use std::ops::Bound::Excluded;
fn main() {
    let m = BTreeMap::from([(1, 1), (5, 5)]);
    println!("{}", m.range(1..5).count());
    let n = m.range((Excluded(5), Excluded(5))).count();
    println!("n={n}");
}
