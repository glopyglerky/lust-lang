// Seeded HashMap/HashSet inserts, removes, entry forms, and iteration
// with a deterministic hasher (W-STD-B): orders and capacities as
// hashbrown gives them, against rustc.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher};
type B = BuildHasherDefault<DefaultHasher>;
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}
fn main() {
    let mut r = Lcg(99);
    let mut m: HashMap<u64, String, B> = HashMap::default();
    let mut s: HashSet<(u8, i16), B> = HashSet::with_capacity_and_hasher(3, BuildHasherDefault::<DefaultHasher>::default());
    for step in 0..400 {
        let k = r.next() % 60;
        match r.next() % 6 {
            0 | 1 => {
                let old = m.insert(k, format!("v{step}"));
                if step % 37 == 0 {
                    println!("ins {k} {old:?} {} {}", m.len(), m.capacity());
                }
            }
            2 => {
                let old = m.remove(&k);
                if step % 41 == 0 {
                    println!("rem {k} {old:?} {}", m.len());
                }
            }
            3 => {
                *m.entry(k).or_insert_with(|| String::new()) += "x";
            }
            4 => {
                s.insert(((k % 7) as u8, (k as i16) - 30));
            }
            _ => {
                s.remove(&(((k % 7) as u8), (k as i16) - 30));
            }
        }
        if step % 97 == 0 {
            let order: Vec<&u64> = m.keys().collect();
            println!("{step} keys {order:?}");
            println!("{step} set {s:?} {}", s.capacity());
        }
    }
    let total: usize = m.values().map(|v| v.len()).sum();
    let mut pairs: Vec<(u64, String)> = m.into_iter().collect();
    pairs.truncate(10);
    println!("{total} {pairs:?}");
    let s2: HashSet<(u8, i16), B> = s.iter().copied().filter(|p| p.0 % 2 == 0).collect();
    println!("{s2:?} {}", s2.capacity());
}
