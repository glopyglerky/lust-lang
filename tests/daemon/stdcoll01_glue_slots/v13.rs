use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<i32> = BTreeSet::new();
    println!("{} {:?}", s.contains(&3), s.get(&-4));
    println!("{:#?} {:?} {:?}", s, s.first(), s.last());
    println!("{}", s.insert(9));
    println!("{}", s.insert(-2));
    println!("{} {:?}", s.contains(&1), s.get(&-1));
    println!("{}", s.insert(6));
    println!("{}", s.remove(&8));
    println!("{}", s.remove(&2));
    let t: BTreeSet<i32> = BTreeSet::from_iter(vec![-2, -2, 1].into_iter()); println!("{} {} {:?}", t.is_subset(&s), s.is_disjoint(&t), t);
    println!("{:#?} {:?} {:?}", s, s.first(), s.last());
    println!("{}", s.insert(2));
}
