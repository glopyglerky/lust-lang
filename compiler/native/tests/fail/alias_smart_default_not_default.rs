// expect: L0612
// rustc E0599: `Rc<T>: Default` needs `T: Default`, which `W` is not.
use std::rc::Rc;
struct W(u8);
type A = Rc<W>;
fn main() {
    let a = true.then(A::default);
    println!("{}", a.is_some());
}
