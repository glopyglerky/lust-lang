// The owning std iterator types by name: vec::IntoIter<T> in a return
// type, an import, and a struct field; hash_map::IntoIter<K, V>.
use std::collections::HashMap;
use std::vec::IntoIter;

fn evens(v: &[u32]) -> std::vec::IntoIter<u32> {
    let out: Vec<u32> = v.iter().copied().filter(|x| x % 2 == 0).collect();
    out.into_iter()
}

fn pairs(m: HashMap<u8, char>) -> std::collections::hash_map::IntoIter<u8, char> {
    m.into_iter()
}

struct Holder {
    it: IntoIter<String>,
}

fn main() {
    let mut e = evens(&[1, 2, 3, 4, 6]);
    println!("{:?} {}", e.next(), e.count());
    let mut m = HashMap::new();
    m.insert(1u8, 'a');
    let p: Vec<(u8, char)> = pairs(m).collect();
    println!("{:?}", p);
    let mut h = Holder { it: vec!["x".to_string(), "y".to_string()].into_iter() };
    println!("{:?}", h.it.next());
}
