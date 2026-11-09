// expect: L0641
// `Weak::new` needs a sized `T` (std's `impl<T> Weak<T>`).
use std::rc::Weak;
trait Tr {
    fn v(&self) -> u32;
}
fn main() {
    let w: Weak<dyn Tr> = Weak::new();
    println!("{}", w.upgrade().is_none());
}
