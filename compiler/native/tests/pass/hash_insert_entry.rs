// Entry::insert_entry and hash_map::OccupiedEntry as std 1.99: values, drop order (the old
// value drops after the new one is stored), remove and remove_entry, and ts-rust's map_insert_new.
use std::collections::HashMap;
#[derive(Debug, PartialEq, Eq, Hash)]
struct K(u32);
impl Drop for K { fn drop(&mut self) { println!("drop K{}", self.0); } }
#[derive(Debug)]
struct V(u32);
impl Drop for V { fn drop(&mut self) { println!("drop V{}", self.0); } }
struct Store { map: HashMap<u32, String> }
impl Store {
    fn map_insert_new(&mut self, key: u32, value: String) -> &mut String {
        self.map.entry(key).insert_entry(value).into_mut()
    }
}
fn main() {
    let mut s = Store { map: HashMap::new() };
    s.map_insert_new(1, "a".into()).push('!');
    s.map_insert_new(1, "b".into()).push('?');
    println!("{:?}", s.map);
    let mut m: HashMap<K, V> = HashMap::new();
    m.insert(K(1), V(10));
    println!("-- occupied");
    let mut o = m.entry(K(1)).insert_entry(V(11));
    println!("after insert_entry");
    println!("show {:?} {:?}", o.key(), o.get());
    o.get_mut().0 += 1;
    let old = o.insert(V(20));
    println!("old {:?}", old);
    drop(old);
    println!("-- vacant");
    let o2 = m.entry(K(2)).insert_entry(V(30));
    println!("{:?} {:?}", o2.key(), o2.get());
    let (k, v) = o2.remove_entry();
    println!("removed {:?} {:?} len {}", k, v, m.len());
    drop((k, v));
    let v = m.entry(K(1)).insert_entry(V(40)).remove();
    println!("removed {:?} len {}", v, m.len());
    let r = m.entry(K(3)).insert_entry(V(50)).into_mut();
    r.0 = 51;
    println!("{:?}", m);
    println!("end");
}
