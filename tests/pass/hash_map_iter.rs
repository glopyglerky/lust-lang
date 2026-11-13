// Map and set iteration in hashbrown bucket order: for loops over &map,
// &mut map, and by value, keys/values/values_mut, collect capacities
// (not TrustedLen), into_keys and into_values, a partly used into_iter.
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

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
    fn finish(&self) -> u64 {
        self.h
    }
}

type B = BuildHasherDefault<Fx>;

fn main() {
    let mut m: HashMap<String, Vec<u32>, B> = HashMap::default();
    for i in 0..20u32 {
        m.entry(format!("w{}", i % 6)).or_default().push(i);
    }
    for (k, v) in &m {
        println!("{} {:?}", k, v);
    }
    for (_, v) in &mut m {
        v.push(100);
    }
    for v in m.values_mut() {
        if v.len() > 4 {
            v.pop();
        }
    }
    let keys: Vec<&String> = m.keys().collect();
    println!("{:?} {}", keys, keys.capacity());
    let total: usize = m.values().map(|v| v.len()).sum();
    println!("{}", total);
    let lens: Vec<(String, usize)> = m.iter().map(|(k, v)| (k.clone(), v.len())).collect();
    println!("{:?}", lens);
    println!("{}", m.iter().count());
    let s: HashSet<u32, B> = {
        let mut s = HashSet::default();
        for x in [5u32, 1, 9, 1, 30, 7] {
            s.insert(x);
        }
        s
    };
    for x in &s {
        print!("{} ", x);
    }
    println!();
    let doubled: Vec<u32> = s.iter().map(|x| x * 2).collect();
    println!("{:?} {}", doubled, s.iter().count());
    let owned: Vec<(String, Vec<u32>)> = m.into_iter().collect();
    println!("{:?}", owned);
    let mut n: HashMap<u8, String, B> = HashMap::default();
    n.insert(1, "one".to_string());
    n.insert(2, "two".to_string());
    n.insert(3, "three".to_string());
    let ks: Vec<u8> = n.clone().into_keys().collect();
    let vs: Vec<String> = n.clone().into_values().collect();
    println!("{:?} {:?}", ks, vs);
    let mut it = n.into_iter();
    println!("{:?}", it.next());
    println!("{}", it.count());
    for x in s {
        print!("{} ", x);
    }
    println!();
}
