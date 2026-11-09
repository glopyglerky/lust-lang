// indexmap 2.14.0 on seeded random operations with integer keys:
// insert, both removes, entry, reserve, shrink, retain, and clear, with
// order, length, and capacity printed as a checksum. A fixed hasher
// keeps capacity deterministic (tombstones depend on hash positions).
use indexmap::IndexMap;
use std::hash::{BuildHasher, Hasher};

#[derive(Default, Clone)]
struct Fnv;

struct FnvHasher(u64);

impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

impl BuildHasher for Fnv {
    type Hasher = FnvHasher;
    fn build_hasher(&self) -> FnvHasher {
        FnvHasher(0xcbf29ce484222325)
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn main() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut m: IndexMap<u64, u64, Fnv> = IndexMap::default();
    let mut sum: u64 = 0;
    for step in 0..20000u64 {
        let k = rng.next() % 300;
        match rng.next() % 10 {
            0..=4 => {
                m.insert(k, step);
            }
            5 => {
                m.shift_remove(&k);
            }
            6 => {
                m.swap_remove(&k);
            }
            7 => {
                *m.entry(k).or_insert(0) += 1;
            }
            8 => {
                if step % 97 == 0 {
                    m.retain(|key, _| key % 5 != 0);
                } else {
                    m.reserve((k % 9) as usize);
                }
            }
            _ => {
                if step % 1009 == 0 {
                    m.clear();
                } else if step % 131 == 0 {
                    m.shrink_to_fit();
                }
            }
        }
        sum = sum.wrapping_mul(31).wrapping_add(m.len() as u64).wrapping_mul(31).wrapping_add(m.capacity() as u64);
        if let Some((a, b)) = m.first() {
            sum = sum.wrapping_add(a ^ b);
        }
    }
    let mut tail = 0u64;
    for (k, v) in &m {
        tail = tail.wrapping_mul(131).wrapping_add(k ^ v);
    }
    println!("{} {} {} {}", m.len(), m.capacity(), sum, tail);
}
