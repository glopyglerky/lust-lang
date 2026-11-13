// Iterator adapters over map and set iterators: copied, cloned, take,
// enumerate, skip, find, any, all, min, max, position, fold, peekable,
// zip, chain, filter over iter_mut, max_by_key over into_iter.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct Fx {
    h: u64,
}

impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.h = (self.h.rotate_left(5) ^ b as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }
    fn write_u32(&mut self, i: u32) {
        self.h = (self.h.rotate_left(5) ^ i as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    fn finish(&self) -> u64 {
        self.h
    }
}

type B = BuildHasherDefault<Fx>;

fn main() {
    let mut m: HashMap<u32, String, B> = HashMap::default();
    for i in 0..12u32 {
        m.insert(i * 3, format!("v{}", i));
    }
    let evens: Vec<u32> = m.keys().copied().filter(|k| k % 2 == 0).collect();
    println!("{:?} {}", evens, evens.capacity());
    let names: Vec<String> = m.values().cloned().take(3).collect();
    println!("{:?}", names);
    for (i, (k, v)) in m.iter().enumerate().skip(9) {
        println!("{} {} {}", i, k, v);
    }
    let found = m.iter().find(|(k, _)| **k > 20);
    if let Some((k, v)) = found {
        println!("{} {}", k, v.len());
    }
    println!("{} {}", m.keys().any(|k| *k == 33), m.values().all(|v| v.starts_with("v")));
    println!("{:?} {:?}", m.keys().max(), m.keys().min());
    println!("{:?}", m.keys().position(|k| *k == 9));
    let s: u32 = m.keys().fold(0, |a, k| a + k);
    println!("{}", s);
    let mut pk = m.keys().peekable();
    let first = match pk.peek() {
        Some(k) => **k,
        None => 0,
    };
    println!("{:?} {:?}", first, pk.next());
    let z: Vec<(&u32, i32)> = m.keys().zip(1..4).collect();
    println!("{:?}", z);
    let set: HashSet<u32, B> = m.keys().map(|k| k % 5).fold(HashSet::default(), |mut acc, x| {
        acc.insert(x);
        acc
    });
    let mut c: Vec<u32> = set.iter().chain(set.iter()).copied().collect();
    c.push(99);
    println!("{:?} {}", c, set.len());
    for (k, v) in m.iter_mut().filter(|(k, _)| **k < 6) {
        v.push_str("!");
        println!("{} {}", k, v);
    }
    let longest = m.into_iter().max_by_key(|(_, v)| v.len());
    println!("{:?}", longest);
}
