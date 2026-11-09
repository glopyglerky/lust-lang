// Regions in a dyn type's trait args (R11): a type argument's region is the
// object's own slot (`dyn Fn(T)`, `dyn Sink<T>`), an elided one in `Fn`
// sugar is late-bound, also on a type with a lifetime (`Fn(W) -> W`), and a
// late-bound object fits a place that fixes the region.
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone, Copy)]
struct W<'a>(&'a u8);
trait Sink<T> {
    fn put(&self, t: T);
}
impl<'v, T> Sink<T> for &'v RefCell<Vec<T>> {
    fn put(&self, t: T) {
        self.borrow_mut().push(t)
    }
}
fn mk<'v, T: 'v>(v: &'v RefCell<Vec<T>>) -> Box<dyn Fn(T) + 'v> {
    Box::new(move |t| v.borrow_mut().push(t))
}
fn sink<'v, T: 'v>(v: &'v RefCell<Vec<T>>) -> Box<dyn Sink<T> + 'v> {
    Box::new(v)
}
fn take<'f, T>(f: Box<dyn Fn(T) + 'f>, t: T) {
    f(t)
}
fn main() {
    let x = 5u8;
    let v: RefCell<Vec<&u8>> = RefCell::new(Vec::new());
    let f = mk(&v);
    f(&x);
    for _ in 0..2 {
        f(&x);
    }
    let s = sink(&v);
    s.put(&x);
    println!("{}", v.borrow().len());
    let g: Box<dyn Fn(W) -> W> = Box::new(|w| w);
    let k = 7u8;
    println!("{}", g(W(&k)).0);
    let h: Rc<dyn Fn(&u8) -> &u8> = Rc::new(|a| a);
    let h2 = h.clone();
    println!("{}", h2(&k));
    let late: Box<dyn Fn(&u8)> = Box::new(|a| println!("late {}", a));
    take(late, &x);
}
