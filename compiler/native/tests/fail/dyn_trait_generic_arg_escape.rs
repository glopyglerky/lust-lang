// expect: L0306
// rustc E0597: a vtable call through `dyn Sink<T>` with `T = &'x u8` binds
// the method's `T` to the object's own (R11).
use std::cell::RefCell;
trait Sink<T> {
    fn put(&self, t: T);
}
struct V<'v, T>(&'v RefCell<Vec<T>>);
impl<'v, T> Sink<T> for V<'v, T> {
    fn put(&self, t: T) {
        self.0.borrow_mut().push(t)
    }
}
fn mk<'v, T: 'v>(v: &'v RefCell<Vec<T>>) -> Box<dyn Sink<T> + 'v> {
    Box::new(V(v))
}
fn main() {
    let v: RefCell<Vec<&u8>> = RefCell::new(Vec::new());
    let s = mk(&v);
    {
        let x = 1u8;
        s.put(&x);
    }
    println!("{}", v.borrow()[0]);
}
