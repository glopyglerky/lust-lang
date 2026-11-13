// A map key with a generic written Hash impl: the hash glue calls its
// `hash::<H>` instance at the key type (gx.generic_hash).
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq)]
struct W<T>(T, u8);

impl<T: Hash> Hash for W<T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h);
    }
}

fn main() {
    let mut m = HashMap::new();
    m.insert(W(1u32, 0), "a");
    println!("{:?}", m.get(&W(1u32, 9)));
}
