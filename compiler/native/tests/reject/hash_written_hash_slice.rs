// expect: L0640
// Valid Rust: a written Hash impl with its own `hash_slice`. A Vec of
// the type hashes through that method, which the glue does not call, so
// lust rejects hashing it.
use std::hash::{BuildHasher, Hash, Hasher, RandomState};

struct B(u8);

impl Hash for B {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_u8(self.0);
    }
    fn hash_slice<H: Hasher>(data: &[B], h: &mut H) {
        h.write_usize(data.len() * 2);
    }
}

fn main() {
    let s = RandomState::new();
    println!("{}", s.hash_one(vec![B(1), B(2)]) == s.hash_one(vec![B(1), B(2)]));
}
