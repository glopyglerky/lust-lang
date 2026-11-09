// Drop order through owning map and set iterators: yielded items, the
// rest at the iterator drop, the builder at into_iter, the value half
// of into_keys and the key half of into_values when taken.
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

#[derive(Default)]
struct LoudBuild;

impl std::hash::BuildHasher for LoudBuild {
    type Hasher = Fx;
    fn build_hasher(&self) -> Fx {
        Fx::default()
    }
}

impl Drop for LoudBuild {
    fn drop(&mut self) {
        println!("drop builder");
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct D(u32);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}

fn main() {
    let mut m: HashMap<D, D, LoudBuild> = HashMap::with_hasher(LoudBuild);
    for i in 0..5 {
        m.insert(D(i), D(100 + i));
    }
    println!("into_iter");
    let mut it = m.into_iter();
    println!("first {:?}", it.next());
    println!("second");
    let second = it.next();
    println!("dropping iterator");
    drop(it);
    println!("second still {:?}", second);
    drop(second);
    let mut k: HashMap<D, D, BuildHasherDefault<Fx>> = HashMap::default();
    for i in 0..3 {
        k.insert(D(i), D(200 + i));
    }
    println!("into_keys");
    let mut ks = k.into_keys();
    println!("key {:?}", ks.next());
    println!("drop keys iterator");
    drop(ks);
    let mut v: HashMap<D, D, BuildHasherDefault<Fx>> = HashMap::default();
    for i in 0..3 {
        v.insert(D(i), D(300 + i));
    }
    println!("into_values");
    for x in v.into_values() {
        println!("value {:?}", x);
        break;
    }
    println!("set");
    let mut s: HashSet<D, BuildHasherDefault<Fx>> = HashSet::default();
    s.insert(D(7));
    s.insert(D(8));
    for x in s {
        println!("got {:?}", x);
    }
    println!("end");
}
