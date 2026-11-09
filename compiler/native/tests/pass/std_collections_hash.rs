// Hash for BTreeMap, BTreeSet, and VecDeque (W-STD-C phase 2):
// the std call sequence on a logging hasher and values through
// a recording hasher and DefaultHasher. The wrapped deque
// hashes in logical order. (As HashMap keys they stay L0640:
// the map glue cannot call generic written Hash impls yet.)
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

struct L(u64);
impl Hasher for L {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, b: &[u8]) {
        for x in b {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*x as u64);
        }
    }
}

fn hv<T: Hash>(t: &T) -> u64 {
    let mut h = L(7);
    t.hash(&mut h);
    h.finish()
}

fn main() {
    let m = BTreeMap::from([(1, String::from("a")), (2, String::from("b"))]);
    let s = BTreeSet::from([3u8, 1, 2]);
    let mut d = VecDeque::from([5u16, 6, 7]);
    d.push_front(4);
    d.pop_back();
    d.push_back(9);
    println!("{} {} {}", hv(&m), hv(&s), hv(&d));
    let mut dh = DefaultHasher::new();
    m.hash(&mut dh);
    println!("{}", dh.finish());
}
