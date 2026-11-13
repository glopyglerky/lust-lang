// Seeded map and set operations on keys with written Hash, PartialEq,
// and Borrow<str> impls (W-STD-B), with deterministic hashers: orders,
// lookups by &str, and capacities as hashbrown gives them, against rustc.
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, DefaultHasher, Hash, Hasher};

type B = BuildHasherDefault<DefaultHasher>;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

// An interned-name shape: hashes and compares as its text.
#[derive(Debug, Clone)]
struct Name {
    text: String,
    uses: u32,
}

impl PartialEq for Name {
    fn eq(&self, o: &Name) -> bool {
        self.text == o.text
    }
}

impl Eq for Name {}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.as_str().hash(state);
    }
}

impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        &self.text
    }
}

// A key that hashes a packed pair, as ts-rust's cache keys do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pair {
    s: u32,
    t: u32,
}

impl Hash for Pair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(((self.s as u64) << 32) | self.t as u64);
    }
}

fn word(n: u64) -> String {
    let syl = ["ka", "to", "ri", "mu", "se"];
    let mut s = String::new();
    let mut n = n;
    loop {
        s.push_str(syl[(n % 5) as usize]);
        n /= 5;
        if n == 0 {
            break;
        }
    }
    s
}

fn main() {
    let mut r = Lcg(7);
    let mut names: HashMap<Name, u64, B> = HashMap::default();
    let mut pairs: HashSet<Pair, B> = HashSet::default();
    for step in 0..500 {
        let k = r.next() % 40;
        let w = word(k);
        match r.next() % 7 {
            0 | 1 => {
                let old = names.insert(Name { text: w.clone(), uses: step }, k);
                if step % 31 == 0 {
                    println!("ins {w} {old:?} {} {}", names.len(), names.capacity());
                }
            }
            2 => {
                let old = names.remove(w.as_str());
                if step % 29 == 0 {
                    println!("rem {w} {old:?} {}", names.len());
                }
            }
            3 => {
                if let Some(v) = names.get_mut(w.as_str()) {
                    *v += 100;
                }
            }
            4 => {
                *names.entry(Name { text: w, uses: 0 }).or_insert(0) += 1;
            }
            5 => {
                pairs.insert(Pair { s: (k % 6) as u32, t: (k / 6) as u32 });
            }
            _ => {
                let p = Pair { s: (k % 6) as u32, t: (k / 6) as u32 };
                pairs.remove(&p);
            }
        }
        if step % 99 == 0 {
            let order: Vec<&str> = names.keys().map(|n| n.text.as_str()).collect();
            println!("{step} names {order:?}");
            let po: Vec<(u32, u32)> = pairs.iter().map(|p| (p.s, p.t)).collect();
            println!("{step} pairs {po:?} {}", pairs.capacity());
        }
    }
    let hits = (0..40).filter(|&k| names.contains_key(word(k).as_str())).count();
    println!("hits {hits}");
    if let Some(n) = names.keys().next() {
        println!("first {} {}", n.text, n.uses);
    }
    println!("{:?}", names.get("kaka"));
}
