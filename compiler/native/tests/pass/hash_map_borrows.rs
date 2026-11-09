// Borrows through map lookups: get_mut then later reads, a lookup key
// that dies before the result, entry or_default chains, RandomState.
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;

fn f<'a>(m: &'a HashMap<String, u32>, k: &str) -> Option<&'a u32> {
    m.get(k)
}

fn g<'a>(s: &'a HashSet<String>, k: &str) -> Option<&'a String> {
    s.get(k)
}

fn main() {
    let mut m: HashMap<String, u32> = HashMap::new();
    m.insert("k".to_string(), 5);
    if let Some(v) = m.get_mut("k") {
        *v += 1;
    }
    let v = {
        let key = String::from("k");
        f(&m, &key)
    };
    let w = {
        let key = String::from("k");
        m.get(&key)
    };
    let mut s = HashSet::new();
    s.insert("q".to_string());
    let x = {
        let key = String::from("q");
        g(&s, &key)
    };
    println!("{:?} {:?} {:?}", v, w, x);
    let mut lists: HashMap<u32, Vec<u32>> = HashMap::new();
    for i in 0..10u32 {
        lists.entry(i % 3).or_default().push(i);
    }
    let e = lists.entry(7).or_insert(Vec::new());
    e.push(1);
    println!("{:?} {:?} {}", lists.get(&0), lists.get(&7), lists.len());
    let mut words: HashMap<String, Vec<u32>> = HashMap::new();
    for i in 0..50u32 {
        words.entry(format!("k{}", i % 7)).or_default().push(i);
    }
    let mut n = 0;
    for i in 0..7 {
        if let Some(v) = words.get(&format!("k{}", i)) {
            n += v.len();
        }
    }
    println!("{} {} {:?}", words.len(), n, words.get("k3"));
    let chars: HashSet<char> = HashSet::with_capacity(3);
    println!("{} {:?} {:?}", chars.capacity(), chars, RandomState::new());
}
