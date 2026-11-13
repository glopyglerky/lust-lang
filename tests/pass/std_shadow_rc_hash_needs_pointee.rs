// Hash is in scope in main, but Rc<Overlay>: Hash needs Overlay: Hash.
use std::hash::Hash;
use std::rc::Rc;
trait FileContent {
    fn hash(&self) -> u128;
}
struct Overlay;
impl FileContent for Overlay {
    fn hash(&self) -> u128 {
        7
    }
}
fn main() {
    let o = Rc::new(Overlay);
    println!("{}", o.hash());

}
fn _uses_hash<T: Hash>(_: T) {}
