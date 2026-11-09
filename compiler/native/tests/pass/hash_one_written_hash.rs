// hash_one of a type with a written Hash impl: the glue calls the
// impl, so K(1) hashes as the byte 2.
use std::hash::{BuildHasher, Hash, Hasher, RandomState};
struct K(u8);
impl Hash for K {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.0 + 1);
    }
}
fn main() {
    let s = RandomState::new();
    println!("{}", s.hash_one(K(1)) == s.hash_one(K(1)));
    let mut h = s.build_hasher();
    h.write_u8(2);
    println!("{}", s.hash_one(K(1)) == h.finish());
}
