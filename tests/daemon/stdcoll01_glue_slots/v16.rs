use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<String> = BTreeSet::new();
    println!("{} {:?}", s.contains(&String::from("dx")), s.get(&String::from("c")));
    println!("{}", s.insert(String::from("byy")));
    let t: BTreeSet<String> = BTreeSet::from_iter(vec![String::from("ax"), String::from("e"), String::from("dx")].into_iter()); println!("{} {} {:?}", t.is_subset(&s), s.is_disjoint(&t), t);
    println!("{}", s.insert(String::from("a")));
    println!("{}", s.insert(String::from("cx")));
    println!("{:?}", s.iter().rev().collect::<Vec<_>>());
}
