// A Hasher that logs every call: the exact write sequence of core's
// Hash impls and derives, hashbrown's empty-table short cut on get,
// and the rehash calls when the table grows.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};

struct Log {
    h: u64,
}

impl Hasher for Log {
    fn write(&mut self, bytes: &[u8]) {
        println!("  write {:?}", bytes);
        for &b in bytes {
            self.h = self.h.wrapping_mul(31).wrapping_add(b as u64);
        }
    }
    fn write_u8(&mut self, i: u8) {
        println!("  write_u8 {}", i);
        self.h = self.h.wrapping_mul(31).wrapping_add(i as u64);
    }
    fn write_usize(&mut self, i: usize) {
        println!("  write_usize {}", i);
        self.h = self.h.wrapping_mul(17).wrapping_add(i as u64);
    }
    fn finish(&self) -> u64 {
        println!("  finish {}", self.h);
        self.h.wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }
}

#[derive(Clone, Copy)]
struct LogBuild;

impl BuildHasher for LogBuild {
    type Hasher = Log;
    fn build_hasher(&self) -> Log {
        println!("  build");
        Log { h: 7 }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum Key {
    A,
    B(u8, String),
    C { x: i32 },
}

fn main() {
    let mut m: HashMap<Key, u32, LogBuild> = HashMap::with_hasher(LogBuild);
    println!("get on empty");
    println!("{:?}", m.get(&Key::A));
    println!("remove on empty");
    println!("{:?}", m.remove(&Key::A));
    println!("insert A");
    m.insert(Key::A, 1);
    println!("insert B");
    m.insert(Key::B(3, "hi".to_string()), 2);
    println!("insert C");
    m.insert(Key::C { x: -5 }, 3);
    println!("insert C again");
    println!("{:?}", m.insert(Key::C { x: -5 }, 4));
    println!("entry B");
    *m.entry(Key::B(3, "hi".to_string())).or_insert(0) += 10;
    println!("entry new");
    *m.entry(Key::B(4, String::new())).or_insert(0) += 10;
    println!("{:?}", m);
    let mut s: HashSet<Vec<u8>, LogBuild> = HashSet::with_capacity_and_hasher(2, LogBuild);
    println!("set inserts");
    for v in [vec![1u8, 2], vec![], vec![3], vec![1, 2], vec![4, 5, 6]] {
        println!("{}", s.insert(v));
    }
    println!("set contains");
    println!("{}", s.contains(&vec![3u8]));
    println!("{}", s.contains(&[9u8][..]));
    println!("{:?}", s);
    let mut t: HashMap<(char, bool, i16), &str, LogBuild> = HashMap::with_hasher(LogBuild);
    t.insert(('x', true, -2), "a");
    t.insert(('y', false, 300), "b");
    println!("{:?}", t);
    let mut u: HashMap<Option<u128>, i128, LogBuild> = HashMap::with_hasher(LogBuild);
    u.insert(Some(1 << 100), -1);
    u.insert(None, 2);
    println!("{:?} {:?}", u.get(&None), u.get(&Some(3)));
}
