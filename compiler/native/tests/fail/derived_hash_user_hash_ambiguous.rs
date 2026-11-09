// expect: L0720
// rustc E0034: Hash is in scope and S derives it, so Hash::hash and U::hash
// both take &self at S's first probe.
use std::hash::Hash;
trait U {
    fn hash(&self, k: u8) -> u64;
}
#[derive(Hash)]
struct S;
impl U for S {
    fn hash(&self, _k: u8) -> u64 {
        7
    }
}
fn main() {
    println!("{}", S.hash(3));
}
