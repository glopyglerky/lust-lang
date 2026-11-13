use std::collections::BTreeSet;
fn main() {
    let mut s: BTreeSet<String> = BTreeSet::new();
    println!("{}", s.insert(String::from("dx")));
    let t: BTreeSet<String> = BTreeSet::from_iter(vec![String::from("eyy"), String::from("fx"), String::from("gyy")].into_iter()); println!("{} {} {:?}", t.is_subset(&s), s.is_disjoint(&t), t);
    println!("{}", s.insert(String::from("c")));
    println!("{}", s.insert(String::from("byy")));
    println!("{}", s.insert(String::from("cyy")));
    println!("{}", s.insert(String::from("gx")));
    println!("{}", s.insert(String::from("gx")));
    println!("{:#?} {:?} {:?}", s, s.first(), s.last());
    let t: BTreeSet<String> = BTreeSet::from_iter(vec![String::from("ex"), String::from("d"), String::from("gyy")].into_iter()); println!("{} {} {:?}", t.is_subset(&s), s.is_disjoint(&t), t);
    println!("{}", s.insert(String::from("ayy")));
    println!("{} {:?}", s.contains(&String::from("b")), s.get(&String::from("e")));
    println!("{}", s.insert(String::from("c")));
}
