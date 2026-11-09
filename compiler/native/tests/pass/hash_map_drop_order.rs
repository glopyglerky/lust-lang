// Drop order of keys and values through insert, entry, remove, clear,
// and the end of scope, for HashMap and HashSet.
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hash, Hasher};

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

#[derive(PartialEq, Eq, Hash, Debug)]
struct K(u32, &'static str);

impl Drop for K {
    fn drop(&mut self) {
        println!("drop K({}, {})", self.0, self.1);
    }
}

#[derive(Debug)]
struct V(i32);

impl Drop for V {
    fn drop(&mut self) {
        println!("drop V({})", self.0);
    }
}

fn _unused<T: Hash>(_: T) {}

fn main() {
    {
        let mut m: HashMap<K, V, BuildHasherDefault<Fx>> = HashMap::default();
        println!("{:?}", m.insert(K(1, "a"), V(10)));
        println!("{:?}", m.insert(K(2, "b"), V(20)));
        println!("insert dup");
        let old = m.insert(K(1, "new"), V(11));
        println!("got {:?}", old);
        drop(old);
        println!("entry occupied");
        let e = m.entry(K(2, "probe"));
        println!("entry made");
        let r = e.or_insert(V(99));
        println!("or_insert done {:?}", r);
        println!("entry vacant");
        let r = m.entry(K(3, "c")).or_insert(V(30));
        r.0 += 1;
        println!("remove");
        let x = m.remove(&K(2, "q"));
        println!("removed {:?}", x);
        println!("remove miss");
        println!("{:?}", m.remove(&K(9, "z")));
        println!("clear");
        m.insert(K(4, "d"), V(40));
        m.clear();
        println!("after clear {}", m.len());
        m.insert(K(5, "e"), V(50));
        m.insert(K(6, "f"), V(60));
        println!("end of scope");
    }
    {
        let mut s: HashSet<K, BuildHasherDefault<Fx>> = HashSet::default();
        s.insert(K(1, "x"));
        println!("set dup {}", s.insert(K(1, "y")));
        println!("set remove {}", s.remove(&K(1, "z")));
        s.insert(K(7, "w"));
        let e = HashMap::<u8, V, BuildHasherDefault<Fx>>::default().entry(3).key().clone();
        println!("key {}", e);
        println!("end of scope 2");
    }
}
