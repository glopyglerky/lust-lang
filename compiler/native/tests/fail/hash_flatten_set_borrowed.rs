// expect: L0303
// rustc E0502: the flatten iterator still borrows `s` when it is changed.
use std::collections::HashSet;
fn main() {
    let mut s: HashSet<u32> = HashSet::new();
    s.insert(1);
    let it = Some(&s).into_iter().flatten();
    s.insert(2);
    println!("{}", it.count());
}
