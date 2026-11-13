// expect: L0610
// rustc E0282: BTreeSet has Extend<T> and Extend<&T>.
use std::collections::BTreeSet;
fn main() {
    let (a, b) = ('a', 'b');
    let mut v = BTreeSet::new();
    v.extend(std::iter::once(&a).chain([&b]));
    println!("{:?}", v);
}
