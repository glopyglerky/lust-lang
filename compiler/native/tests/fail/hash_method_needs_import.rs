// expect: L0104
// rustc E0599: only the `Hash` derive macro is in the prelude; `.hash()` needs `use std::hash::Hash`.
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
struct S;
impl std::hash::Hash for S {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_u8(1)
    }
}
fn main() {
    let mut d = DefaultHasher::new();
    S.hash(&mut d);
    println!("{}", d.finish());
}
