// expect: L0612
// rustc E0277: `f64` is not `Ord`, so there is no `FromIterator` impl.
use std::collections::BTreeSet;
fn main() {
    let s: BTreeSet<f64> = vec![1.0].into_iter().collect();
    println!("{}", s.len());
}
