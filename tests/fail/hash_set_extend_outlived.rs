// expect: L0306
// rustc E0597: extend stores borrows of `v` elements in the set.
use std::collections::HashSet;

fn main() {
    let mut s: HashSet<&u32> = HashSet::new();
    {
        let v = vec![1u32, 2];
        s.extend(v.iter());
    }
    println!("{:?}", s.len());
}
