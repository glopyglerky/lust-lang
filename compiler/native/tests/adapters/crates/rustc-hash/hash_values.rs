// rustc-hash 2.1.2: FxHasher values on upstream's own test vectors
// (unsigned, signed, byte strings) plus seeded hashers and strings,
// through hash_one, Hash::hash, and direct writes.
use rustc_hash::{FxBuildHasher, FxHasher, FxSeededState};
use std::hash::{BuildHasher, Hash, Hasher};

struct HashBytes(&'static [u8]);
impl Hash for HashBytes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0);
    }
}

static BYTES: [&[u8]; 7] = [b"", b"\0", b"\0\0\0\0\0\0", b"\x01", b"\x02", b"uwu", b"These are some bytes for testing rustc_hash."];

fn main() {
    println!("{} {} {} {}", FxBuildHasher.hash_one(0u8), FxBuildHasher.hash_one(1u8), FxBuildHasher.hash_one(100u8), FxBuildHasher.hash_one(u8::MAX));
    println!("{} {} {}", FxBuildHasher.hash_one(1u16), FxBuildHasher.hash_one(u16::MAX), FxBuildHasher.hash_one(u32::MAX));
    println!("{} {} {}", FxBuildHasher.hash_one(1u64), FxBuildHasher.hash_one(u64::MAX), FxBuildHasher.hash_one(u128::MAX));
    println!("{} {} {}", FxBuildHasher.hash_one(1u128), FxBuildHasher.hash_one(100u128), FxBuildHasher.hash_one(usize::MAX));
    println!("{} {} {}", FxBuildHasher.hash_one(i8::MIN), FxBuildHasher.hash_one(i16::MAX), FxBuildHasher.hash_one(i32::MIN));
    println!("{} {} {}", FxBuildHasher.hash_one(i64::MIN), FxBuildHasher.hash_one(i128::MAX), FxBuildHasher.hash_one(isize::MAX));
    for b in BYTES {
        println!("{}", FxBuildHasher.hash_one(HashBytes(b)));
    }
    println!("{} {}", FxBuildHasher.hash_one("hello"), FxBuildHasher.hash_one(String::from("a longer string for the bulk path!")));
    println!("{} {}", FxBuildHasher.hash_one((1u32, 'x', true)), FxBuildHasher.hash_one(vec![1u8, 2, 3]));
    let mut h = FxHasher::default();
    h.write_u32(7);
    h.write(b"abc");
    h.write_usize(9);
    println!("{}", h.finish());
    let mut s = FxHasher::with_seed(42);
    "seeded".hash(&mut s);
    println!("{}", s.finish());
    println!("{}", FxSeededState::with_seed(3).hash_one(77u64));
    let d = FxHasher::default();
    println!("{} {}", d.finish(), FxHasher::default().finish());
}
