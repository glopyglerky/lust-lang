// expect: L0110
// rustc E0276: the impl asks more of `H` than the trait does.
use std::hash::{Hash, Hasher};
struct S(u8);
impl Hash for S {
    fn hash<H: Hasher + Clone>(&self, state: &mut H) {
        state.write_u8(self.0);
    }
}
fn main() {
    println!("{}", S(1).0);
}
