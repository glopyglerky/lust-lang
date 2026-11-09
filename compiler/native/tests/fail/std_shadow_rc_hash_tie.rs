// expect: L0720
// Rc<S>: Hash holds (S derives Hash) and Hash is in scope, so std's
// Hash::hash on Rc<S> ties with U::hash: rustc E0034 (both take &self
// at the Rc step).
use std::hash::Hash;
use std::rc::Rc;
trait U {
    fn hash(&self) -> u64;
}
#[derive(Hash)]
struct S;
impl U for Rc<S> {
    fn hash(&self) -> u64 {
        7
    }
}
fn main() {
    let x = Rc::new(S);
    println!("{}", x.hash());
}
