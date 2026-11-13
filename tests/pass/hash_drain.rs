// HashMap::drain and HashSet::drain (R20): iteration order, drop order of the rest, capacity kept, tombstones cleared.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type H = BuildHasherDefault<DefaultHasher>;
#[derive(Debug, PartialEq, Eq, Hash)]
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
fn main() {
    let mut m: HashMap<u32, String, H> = HashMap::default();
    for i in 0..20 { m.insert(i * 7, format!("v{i}")); }
    let order: Vec<u32> = m.keys().copied().collect();
    let cap = m.capacity();
    let drained: Vec<(u32, String)> = m.drain().collect();
    println!("{}", order == drained.iter().map(|p| p.0).collect::<Vec<_>>());
    println!("{:?}", drained.iter().take(3).collect::<Vec<_>>());
    println!("{} {} {}", m.len(), m.capacity() == cap, m.is_empty());
    m.insert(1, "x".into());
    println!("{:?}", m);
    let mut dm: HashMap<u32, D, H> = HashMap::default();
    for i in 0..6 { dm.insert(i, D(i)); }
    {
        let mut d = dm.drain();
        println!("len {} {:?}", d.len(), d.size_hint());
        let a = d.next();
        println!("first {:?} {}", a, d.len());
        println!("dropping drain");
    }
    println!("after {} {}", dm.len(), dm.capacity());
    let mut s: HashSet<String, H> = HashSet::default();
    for w in ["a", "b", "c", "d"] { s.insert(w.to_string()); }
    let it: Vec<String> = s.iter().cloned().collect();
    let ds: Vec<String> = s.drain().collect();
    println!("{} {}", it == ds, s.len());
    let mut e: HashMap<u8, u8> = HashMap::new();
    println!("{} {}", e.drain().count(), e.capacity());
    let mut t: HashMap<u32, u32, H> = HashMap::default();
    for i in 0..14 { t.insert(i, i); }
    for i in 0..14 { t.remove(&i); }
    let before = t.capacity();
    t.drain();
    println!("{} {}", before, t.capacity());
    let mut n = 0;
    for (k, v) in dm.drain() { n += k + v.0; }
    println!("{n}");
    let mut r: HashMap<u32, D, H> = HashMap::default();
    r.insert(5, D(5));
    let x = &mut r;
    let c = x.drain().count();
    println!("{} {}", c, r.len());
}
