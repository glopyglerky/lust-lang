// `x.hash(&mut h)` for core and derived Hash, `BuildHasher::hash_one`,
// and Hasher methods on a concrete hasher (DefaultHasher, a user
// hasher's provided write_*), in generic and plain code (W-STD-B).
use std::collections::HashMap;
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher, RandomState};

struct F(u64);
impl Hasher for F {
    fn write(&mut self, b: &[u8]) {
        for x in b {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*x as u64);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}
#[derive(Default, Clone)]
struct FB;
impl BuildHasher for FB {
    type Hasher = F;
    fn build_hasher(&self) -> F {
        F(7)
    }
}
#[derive(Hash, PartialEq, Eq)]
struct Key {
    a: u32,
    b: String,
    c: Option<u8>,
}
struct Manual(u16);
impl Hash for Manual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        "tag".hash(state);
        state.write_u8(1);
    }
}
fn generic<Q: ?Sized + Hash, S: BuildHasher>(s: &S, q: &Q) -> u64 {
    s.hash_one(q)
}
fn into<T: Hash, H: Hasher>(t: &T, h: &mut H) {
    t.hash(h);
    h.write_u16(9);
}
fn main() {
    let mut h = F(0);
    h.write_u32(7);
    h.write(b"ab");
    h.write_i64(-3);
    h.write_usize(5);
    println!("{}", h.finish());
    let mut d = DefaultHasher::new();
    d.write_u8(3);
    42u64.hash(&mut d);
    "s".hash(&mut d);
    println!("{}", d.finish());
    let k = Key { a: 1, b: "x".to_string(), c: Some(2) };
    let mut f = F(1);
    k.hash(&mut f);
    Manual(5).hash(&mut f);
    println!("{}", f.finish());
    println!("{} {}", FB.hash_one(&k), generic(&FB, "str"));
    let bd: BuildHasherDefault<DefaultHasher> = BuildHasherDefault::default();
    println!("{} {}", bd.hash_one(12u8), generic(&bd, &[1u32, 2]));
    let mut g = F(2);
    into(&(1u8, 2i32), &mut g);
    println!("{}", g.finish());
    let rs = RandomState::new();
    println!("{}", rs.hash_one(1) == rs.hash_one(1));
    let mut m: HashMap<u8, u8, FB> = HashMap::default();
    m.insert(1, 2);
    println!("{:?}", m.get(&1));
}
