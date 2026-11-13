// HashMap and HashSet over a user Hasher through BuildHasherDefault:
// entry, get, remove, insert, clear, clone, capacity, and Debug in
// hashbrown bucket order.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
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

type FxBuild = BuildHasherDefault<Fx>;

fn main() {
    let mut m: HashMap<String, u32, FxBuild> = HashMap::default();
    for (i, w) in ["a", "bb", "ccc", "a", "dd", "bb", "a"].iter().enumerate() {
        let c = m.entry(w.to_string()).or_insert(0);
        *c += i as u32;
    }
    println!("{} {}", m.len(), m.capacity());
    println!("{:?}", m.get("a"));
    println!("{:?}", m.get("zz"));
    println!("{}", m.contains_key("dd"));
    println!("{:?}", m.remove("bb"));
    println!("{:?}", m.insert("q".to_string(), 7));
    println!("{:?}", m.insert("q".to_string(), 8));
    if let Some(v) = m.get_mut("q") {
        *v *= 10;
    }
    println!("{:?}", m);
    let mut s: HashSet<u32, FxBuild> = HashSet::default();
    for i in [5u32, 3, 9, 3, 1000, 5] {
        println!("{}", s.insert(i));
    }
    println!("{:?} {}", s, s.contains(&9));
    println!("{} {}", s.remove(&9), s.remove(&9));
    println!("{:?} {}", s, s.len());
    let e: HashMap<u8, Vec<String>, FxBuild> = HashMap::with_capacity_and_hasher(10, BuildHasherDefault::<Fx>::default());
    println!("{} {}", e.capacity(), e.is_empty());
    let mut big: HashMap<u32, u32, FxBuild> = HashMap::default();
    for i in 0..40u32 {
        *big.entry(i * 7 % 13).or_default() += i;
    }
    println!("{:?}", big);
    let c = big.clone();
    big.clear();
    println!("{} {} {:?}", big.len(), big.capacity(), c.get(&3));
}
