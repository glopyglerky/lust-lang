// `b.build_hasher()` and `b.hash_one(x)` on a user `BuildHasher`
// value (rustc accepts both; lust used to reject the first), a
// written `hash_one` (it runs as written, not as the provided
// body), a seeded builder, a builder for one generic instance,
// and the type-path and qualified forms. Std's own builders
// are in `hash_build_hasher.rs`.
use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
struct F(u64);
impl Hasher for F {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = self.0.wrapping_mul(31).wrapping_add(x as u64);
        }
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

struct Seeded(u64);
impl BuildHasher for Seeded {
    type Hasher = F;
    fn build_hasher(&self) -> F {
        F(self.0)
    }
    fn hash_one<T: std::hash::Hash>(&self, _x: T) -> u64 {
        self.0 + 1000
    }
}

struct GB<T>(T);
impl BuildHasher for GB<u8> {
    type Hasher = F;
    fn build_hasher(&self) -> F {
        F(self.0 as u64)
    }
}

fn main() {
    let b = FB;
    let mut h = b.build_hasher();
    h.write_u8(3);
    println!("{} {}", h.finish(), b.hash_one(9u8));
    let r = &FB;
    let mut h2 = r.build_hasher();
    h2.write_u8(3);
    println!("{}", h2.finish());

    let s = Seeded(40);
    println!("{} {}", s.build_hasher().finish(), s.hash_one(2u8));
    println!("{}", Seeded::build_hasher(&s).finish());
    println!("{}", <Seeded as BuildHasher>::hash_one(&s, 2u8));
    println!("{}", FB::hash_one(&FB, 3u8));

    let g = GB(9u8);
    println!("{} {}", g.build_hasher().finish(), g.hash_one(1u8));
}
