// panic: range start is greater than range end in BTreeSet
use std::collections::BTreeSet;
fn main() {
    let s = BTreeSet::from([1, 5]);
    println!("{}", s.range(1..5).count());
    let n = s.range(5..1).count();
    println!("n={n}");
}
