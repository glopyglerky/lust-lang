// expect: L0610
// rustc E0282: HashSet has Extend<T> and Extend<&T>.
use std::collections::HashSet;
fn main() {
    let src = vec![1u8, 2];
    let mut v = HashSet::new();
    v.extend(&src);
    println!("{}", v.len());
}
