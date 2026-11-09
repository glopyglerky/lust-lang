// expect: L0311
// rustc E0505: the set holds borrows of `v`, so `v` cannot move.
use std::collections::BTreeSet;
fn main() {
    let v = vec![String::new()];
    let s: BTreeSet<&String> = v.iter().collect();
    drop(v);
    println!("{}", s.len());
}
