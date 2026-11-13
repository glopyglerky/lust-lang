use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<&str> = BTreeSet::new();
    println!("{}", s.insert("k"));
    println!("{:?}", s.iter().rev().collect::<Vec<_>>());
    println!("{} {:?}", s.contains(&"m"), s.get(&"a"));
    println!("{}", s.insert("b"));
    println!("{}", s.insert("zz"));
    println!("{:?}", s.iter().rev().collect::<Vec<_>>());
}
