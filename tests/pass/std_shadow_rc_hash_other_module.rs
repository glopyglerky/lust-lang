// std's Hash::hash is not a candidate: Overlay has no Hash impl, and
// Hash is not in scope here. rustc picks FileContent::hash.
use std::rc::Rc;
mod other {
    #[allow(unused_imports)]
    use std::hash::Hash;
}
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
    let r = &o;
    println!("{}", r.hash());
    let d: Rc<dyn FileContent> = Rc::new(Overlay);
    println!("{}", d.hash());
}
