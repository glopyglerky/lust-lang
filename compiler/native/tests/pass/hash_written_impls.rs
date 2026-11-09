// Written Hash impls through the hash glue: map and set keys, derived
// impls around a written one, two hashers, and iteration order with
// deterministic hashers (the order rustc prints).
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    id: u32,
    note: String,
}

// Hashes only the id; equal keys have equal ids.
impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Dot,
    Line(u8),
    Box { w: u16, h: u16 },
}

impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Shape::Dot => state.write_u8(7),
            Shape::Line(n) => {
                state.write_u8(1);
                n.hash(state);
            }
            Shape::Box { w, h } => state.write_u64(((*w as u64) << 16) | *h as u64),
        }
    }
}

// A written impl that calls another written one.
#[derive(Debug, PartialEq, Eq)]
struct Pair(Key, Shape);

impl Hash for Pair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
        self.0.hash(state);
        state.write_usize(self.0.note.len());
    }
}

// A derived impl around written ones.
#[derive(Debug, PartialEq, Eq, Hash)]
struct Wrap {
    k: Key,
    s: Option<Shape>,
    v: Vec<Shape>,
}

// Case-insensitive text: written Hash and PartialEq agree.
#[derive(Debug)]
struct Ci(String);

impl PartialEq for Ci {
    fn eq(&self, o: &Ci) -> bool {
        self.0.to_lowercase() == o.0.to_lowercase()
    }
}

impl Eq for Ci {}

impl Hash for Ci {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.0.bytes() {
            state.write_u8(if b >= b'A' && b <= b'Z' { b + 32 } else { b });
        }
    }
}

#[derive(Default)]
struct Fnv(u64);

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x100000001b3);
        }
    }
}

type Det = BuildHasherDefault<DefaultHasher>;
type FnvB = BuildHasherDefault<Fnv>;

fn key(id: u32, note: &str) -> Key {
    Key { id, note: note.to_string() }
}

fn main() {
    let mut m: HashMap<Key, u32, Det> = HashMap::default();
    for i in 0..12 {
        m.insert(key(i * 7, "n"), i);
    }
    println!("{:?}", m.keys().map(|k| k.id).collect::<Vec<_>>());
    println!("{:?}", m.get(&key(14, "n")));

    let mut f: HashMap<Key, u32, FnvB> = HashMap::default();
    for i in 0..12 {
        f.insert(key(i * 5, "x"), i);
    }
    println!("{:?}", f.keys().map(|k| k.id).collect::<Vec<_>>());

    let shapes: HashSet<Shape, Det> = [Shape::Dot, Shape::Line(3), Shape::Box { w: 2, h: 9 }, Shape::Line(4)].into_iter().collect();
    println!("{:?}", shapes.iter().collect::<Vec<_>>());
    println!("{}", shapes.contains(&Shape::Line(3)));

    let mut pm: HashMap<Pair, &str, FnvB> = HashMap::default();
    pm.insert(Pair(key(1, "ab"), Shape::Dot), "p1");
    pm.insert(Pair(key(2, "c"), Shape::Line(9)), "p2");
    println!("{:?} {:?}", pm.get(&Pair(key(1, "ab"), Shape::Dot)), pm.len());

    let mut wm: HashMap<Wrap, u8, Det> = HashMap::default();
    for i in 0..5u8 {
        wm.insert(Wrap { k: key(i as u32, "w"), s: if i % 2 == 0 { Some(Shape::Line(i)) } else { None }, v: vec![Shape::Dot; i as usize] }, i);
    }
    println!("{:?}", wm.values().collect::<Vec<_>>());

    let mut ci: HashMap<Ci, u32> = HashMap::new();
    *ci.entry(Ci("Hello".into())).or_insert(0) += 1;
    *ci.entry(Ci("HELLO".into())).or_insert(0) += 1;
    *ci.entry(Ci("world".into())).or_insert(0) += 1;
    println!("{} {:?}", ci.len(), ci.get(&Ci("hello".into())));

    let det = BuildHasherDefault::<DefaultHasher>::default();
    println!("{}", det.hash_one(key(5, "q")) == det.hash_one(5u32));
    println!("{}", det.hash_one(&key(5, "q")) == det.hash_one(5u32));
    let fb = BuildHasherDefault::<Fnv>::default();
    println!("{} {}", fb.hash_one(Shape::Box { w: 1, h: 2 }), fb.hash_one((Shape::Dot, key(3, ""))));
    let mut h = fb.build_hasher();
    Pair(key(4, "zz"), Shape::Line(1)).hash(&mut h);
    vec![key(1, ""), key(2, "")].hash(&mut h);
    Some(Shape::Dot).hash(&mut h);
    println!("{}", h.finish());

    let a: HashSet<Key, Det> = (0..4).map(|i| key(i, "a")).collect();
    let b: HashSet<Key, Det> = (0..4).rev().map(|i| key(i, "a")).collect();
    println!("{}", a == b);
}
