// expect: L0110
// `hash_slice` takes `data: &[Self]`, not `&self` (rustc: E0185).
use std::hash::{Hash, Hasher};

struct B(u8);

impl Hash for B {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_u8(self.0);
    }
    fn hash_slice<H: Hasher>(&self, h: &mut H) {
        h.write_u8(0);
    }
}

fn main() {
    let _ = B(1);
}
