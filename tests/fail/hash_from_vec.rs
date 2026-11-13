// expect: L0612
// rustc E0277: HashSet has no `From<Vec<u8>>` (collect the Vec instead).
use std::collections::HashSet;

fn main() {
    let s = HashSet::from(vec![1u8]);
    println!("{}", s.len());
}
