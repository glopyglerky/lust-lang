// A raw pointer to a dyn value kept in a binding (tf-fatptr; a lust
// reject under the first, read-at-once R14).
use std::rc::Rc;
trait Host {}
struct H;
impl Host for H {}
fn main() {
    let a: Rc<dyn Host> = Rc::new(H);
    let p = Rc::as_ptr(&a);
    println!("{}", std::mem::size_of_val(&p));
}
