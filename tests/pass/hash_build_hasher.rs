// `build_hasher` on std's builders: RandomState builds a DefaultHasher
// with its keys, BuildHasherDefault<H> an `H::default()`. The hasher
// owns no borrow of its builder.
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher, RandomState};

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

fn fresh() -> DefaultHasher {
    let s = RandomState::new();
    s.build_hasher()
}

fn main() {
    let b: BuildHasherDefault<Fnv> = BuildHasherDefault::default();
    let mut h = b.build_hasher();
    h.write_u32(7);
    "ab".hash(&mut h);
    println!("{}", h.finish());
    println!("{}", b.hash_one(7u32));

    let d = BuildHasherDefault::<DefaultHasher>::default();
    let mut g = d.build_hasher();
    g.write_u8(1);
    let mut g2 = DefaultHasher::new();
    g2.write_u8(1);
    println!("{}", g.finish() == g2.finish());

    let r = RandomState::new();
    let one = {
        let mut x = r.build_hasher();
        42u64.hash(&mut x);
        x.finish()
    };
    println!("{}", one == r.hash_one(42u64));
    let rr = &r;
    let mut y = rr.build_hasher();
    let mut y2 = rr.build_hasher();
    drop(r);
    y.write_u8(3);
    y2.write_u8(3);
    println!("{}", y.finish() == y2.finish());
    let mut f = fresh();
    f.write_u8(3);
    println!("{}", f.finish() == f.finish());
}
