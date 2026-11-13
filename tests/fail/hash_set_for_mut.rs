// expect: L0113
// rustc E0277: there is no IntoIterator for &mut HashSet.
use std::collections::HashSet;

fn main() {
    let mut s: HashSet<u32> = HashSet::new();
    s.insert(1);
    for x in &mut s {
        println!("{}", x);
    }
}
