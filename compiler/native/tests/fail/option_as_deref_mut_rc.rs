// expect: L0612
// `Rc<T>` has no `DerefMut`, so `Option<Rc<T>>` has no `as_deref_mut`.
use std::rc::Rc;
fn main() {
    let mut o: Option<Rc<i32>> = Some(Rc::new(5));
    if let Some(v) = o.as_deref_mut() {
        *v += 1;
    }
    println!("{:?}", o);
}
