// `entries(iter)` on DebugMap, DebugSet, and DebugList (W-STD-B): one
// entry per item, also under `{:#?}` and a width.
use std::fmt;

struct Bag(Vec<(String, u8)>);
impl fmt::Debug for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter().map(|(k, v)| (k, v))).finish()
    }
}
struct Set(Vec<i32>);
impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.0.iter()).entries(&[7, 8]).finish()
    }
}
struct List(Vec<String>);
impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entry(&0).entries(self.0.clone()).finish()
    }
}
fn main() {
    let b = Bag(vec![("a".to_string(), 1), ("b".to_string(), 2)]);
    println!("{:?}", b);
    println!("{:#?}", b);
    println!("{:?} {:?}", Set(vec![3, 1]), List(vec!["x".into(), "y".into()]));
    println!("{:#?}", Set(vec![]));
    println!("{:>6?}", List(vec![]));
}
