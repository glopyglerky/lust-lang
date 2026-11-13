// expect: L0306
// rustc E0597: the set's key holds a Cell that the predicate writes &t into.
use std::cell::Cell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
struct K<'a>(u8, Cell<&'a str>);
impl PartialEq for K<'_> { fn eq(&self, o: &Self) -> bool { self.0 == o.0 } }
impl Eq for K<'_> {}
impl Hash for K<'_> { fn hash<H: Hasher>(&self, h: &mut H) { self.0.hash(h) } }
fn main() {
    let mut s = HashSet::new();
    s.insert(K(1, Cell::new("a")));
    {
        let t = String::from("x");
        s.retain(|k| { k.1.set(&t); true });
    }
    println!("{}", s.iter().next().unwrap().1.get());
}
