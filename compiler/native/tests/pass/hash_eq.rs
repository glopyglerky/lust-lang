// HashMap and HashSet `==` (W-STD-B): std's PartialEq (lengths, then each
// element of the left table found in the right, values compared left
// first), also through a `T: PartialEq` bound, assert_eq!, assert_ne!.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type B = BuildHasherDefault<DefaultHasher>;

#[derive(Debug)]
struct Loud(u8);
impl PartialEq for Loud {
    fn eq(&self, o: &Loud) -> bool {
        println!("eq {} {}", self.0, o.0);
        self.0 == o.0
    }
}
fn same<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}
fn main() {
    let a: HashMap<&str, i32> = [("x", 1), ("y", 2)].into_iter().collect();
    let mut b = HashMap::new();
    b.insert("y", 2);
    b.insert("x", 1);
    println!("{} {}", a == b, a != b);
    b.insert("z", 3);
    println!("{}", a == b);
    b.remove("z");
    b.insert("x", 9);
    println!("{} {}", a == b, same(&a, &b));
    let s: HashSet<u8> = [1, 2, 3].into_iter().collect();
    let t: HashSet<u8> = [3, 2, 1].into_iter().collect();
    println!("{} {}", s == t, same(&s, &HashSet::new()));
    let mut l: HashMap<u8, Loud, B> = HashMap::default();
    l.insert(1, Loud(5));
    l.insert(2, Loud(6));
    let mut r: HashMap<u8, Loud, B> = HashMap::default();
    r.insert(2, Loud(6));
    r.insert(1, Loud(7));
    println!("{}", l == r);
    assert_eq!(s, t);
    let e: HashMap<u8, u8> = HashMap::new();
    assert_ne!(e, [(1, 1)].into_iter().collect());
    println!("done");
}
