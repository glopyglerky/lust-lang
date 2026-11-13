use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<i32> = BTreeSet::new();
    println!("{}", s.insert(2));
    println!("{}", s.insert(1));
    println!("{} {:?}", s.contains(&6), s.get(&6));
    let t: BTreeSet<i32> = BTreeSet::from_iter(vec![13, -5, 5].into_iter()); println!("{} {} {:?}", t.is_subset(&s), s.is_disjoint(&t), t);
    println!("{} {:?}", s.contains(&1), s.get(&8));
    println!("{}", s.insert(11));
    println!("{}", s.insert(4));
    println!("{}", s.insert(9));
    println!("{:?}", s.iter().rev().collect::<Vec<_>>());
}
