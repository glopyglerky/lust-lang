// collect into HashMap and HashSet (FromIterator), extend (the empty
// table reserves the hint, a full one half of it), and reserve.
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
    fn finish(&self) -> u64 {
        self.h
    }
}

type B = BuildHasherDefault<Fx>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum Kw {
    If,
    Else,
    While,
}

const WORDS: [(&str, Kw); 3] = [("if", Kw::If), ("else", Kw::Else), ("while", Kw::While)];

fn main() {
    let m: HashMap<u32, String, B> = (0..10u32).map(|i| (i * 7 % 11, format!("x{}", i))).collect();
    println!("{:?} {} {}", m, m.len(), m.capacity());
    let s: HashSet<char, B> = "hello world".chars().collect();
    println!("{:?} {}", s, s.capacity());
    let mut kw: HashMap<&str, Kw, B> = HashMap::default();
    kw.extend(WORDS.iter().copied());
    println!("{:?} {}", kw, kw.capacity());
    let mut names: HashSet<String, B> = HashSet::default();
    names.insert("a".to_string());
    let mut more: HashMap<String, bool, B> = HashMap::default();
    for w in ["b", "c", "a", "d"] {
        more.insert(w.to_string(), true);
    }
    names.extend(more.into_keys());
    println!("{:?} {}", names, names.capacity());
    let mut big: HashSet<u32, B> = HashSet::default();
    big.reserve(10);
    println!("{}", big.capacity());
    big.extend(vec![1u32, 2, 3, 2, 1]);
    big.extend(vec![9u32, 8].into_iter());
    println!("{:?} {}", big, big.capacity());
    let pairs: Vec<(u8, u8)> = vec![(1, 2), (3, 4), (1, 5)];
    let pm: HashMap<u8, u8, B> = pairs.into_iter().collect();
    println!("{:?}", pm);
    let from_set: Vec<u32> = big.into_iter().collect();
    println!("{:?}", from_set);
}
