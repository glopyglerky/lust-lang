// Entry::or_insert_with, or_insert_with_key, and and_modify take FnOnce
// closures, as std does: a closure may move its captures out.
use std::collections::HashMap;
struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let mut m: HashMap<u8, Vec<u8>> = HashMap::new();
    let v = vec![1u8];
    m.entry(1).or_insert_with(move || v).push(2);
    let mut n: HashMap<u8, u8> = HashMap::new();
    let d = D(7);
    n.entry(1).or_insert_with(move || {
        let x = d;
        x.0
    });
    let d2 = D(8);
    n.entry(1).or_insert_with(move || d2.0);
    println!("{:?} {:?}", m, n);
    let mut k: HashMap<String, usize> = HashMap::new();
    let tag = String::from("t");
    k.entry("abc".to_string()).or_insert_with_key(move |key| key.len() + tag.len());
    let mut w: HashMap<u8, Vec<String>> = HashMap::new();
    w.insert(1, vec![]);
    let s = String::from("s");
    w.entry(1).and_modify(move |v| v.push(s)).or_default();
    println!("{:?} {:?}", k, w);
}
