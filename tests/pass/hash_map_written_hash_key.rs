// A key with a written Hash impl: the hash glue calls the impl's
// `hash::<H>` instance, so K(0) and K(1) share a hash (still unequal).
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug)]
struct K(u32);

impl Hash for K {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_u32(self.0 / 2);
    }
}

fn main() {
    let mut m = HashMap::new();
    m.insert(K(1), 1);
    println!("{:?}", m.get(&K(0)));
}
