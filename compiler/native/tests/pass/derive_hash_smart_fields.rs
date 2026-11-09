// Derived Hash that reaches Arc, Box<[T]>, VecDeque, BTreeMap, BTreeSet, and
// fn pointer fields: the hash glue hashes an Arc as its value, a boxed slice
// as a slice, calls the std collections' generic Hash impls, and hashes a fn
// pointer by address (hunt4: Arc and Box<[T]> recursed forever, VecDeque and
// fn pointers passed check and failed the build). Box<[T]> also orders as
// its slice.
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn inc(x: u8) -> u8 {
    x + 1
}

#[derive(Hash)]
struct A {
    f: Arc<u8>,
    s: Arc<str>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
enum E {
    V(Arc<Vec<u16>>),
    B(Box<[u16]>),
    N,
}

#[derive(Hash, PartialEq, Eq)]
struct C {
    q: VecDeque<String>,
    m: BTreeMap<u8, Arc<u8>>,
    s: BTreeSet<i32>,
}

#[derive(Hash, Clone, Copy)]
struct F {
    f: fn(u8) -> u8,
}

#[derive(Hash)]
struct W<T> {
    t: T,
}

fn main() {
    println!("{}", hash_of(&A { f: Arc::new(5), s: Arc::from("ab") }));
    println!("{}", hash_of(&vec![Arc::new(1u32), Arc::new(2)]));
    println!("{}", hash_of(&[E::V(Arc::new(vec![1, 2])), E::B(vec![3u16, 4].into_boxed_slice()), E::N]));
    let (x, y) = (E::B(vec![1u16, 2].into_boxed_slice()), E::B(vec![1u16, 3].into_boxed_slice()));
    println!("{:?} {:?} {}", x.partial_cmp(&y), y.cmp(&x), x < y);
    let mut q = VecDeque::new();
    q.push_back(String::from("b"));
    q.push_front(String::from("a"));
    let c = C { q, m: BTreeMap::from([(2, Arc::new(7)), (1, Arc::new(9))]), s: BTreeSet::from([3, -1]) };
    println!("{}", hash_of(&c));
    println!("{}", hash_of(&W { t: VecDeque::from(vec![1u8, 2]) }));
    let mut set = HashSet::new();
    set.insert(c);
    println!("{}", set.len());
    // A fn pointer hashes its address: the value differs per binary, so
    // only compare.
    let f = F { f: inc };
    let g = f;
    println!("{} {}", hash_of(&f) == hash_of(&g), (f.f)(1));
    let p: fn(u8) -> u8 = inc;
    let mut h = DefaultHasher::new();
    p.hash(&mut h);
    println!("{}", h.finish() == hash_of(&p));
    println!("{}", hash_of(&VecDeque::from(vec![p])) == hash_of(&VecDeque::from(vec![p])));
}
