// expect: L0643
// Hash is in scope and Box<S>: Hash holds, so Hash::hash at the Box step
// comes before S::hash at the S step: rustc picks std's (E0061 here).
use std::hash::Hash;
#[derive(Hash)]
struct S(u32);
impl S {
    fn hash(&self) -> u32 {
        970
    }
}
fn main() {
    let r = Box::new(S(2));
    println!("{:?}", r.hash());
}
