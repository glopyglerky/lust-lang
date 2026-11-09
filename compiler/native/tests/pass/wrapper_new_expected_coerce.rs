// `Mutex::new(x)` (and RwLock, RefCell, Cell, Rc, Arc, Box) where the
// wrapper type is expected: the argument coerces to the expected value
// type, as rustc's expected inputs make it (gostd context `after_func`).
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
trait Show {
    fn show(&self) -> u8;
}
struct K;
impl Show for K {
    fn show(&self) -> u8 {
        4
    }
}
fn one() -> u8 {
    1
}
struct Hooks {
    f: Mutex<Option<Box<dyn FnOnce() -> u8 + Send>>>,
}
fn mk() -> RefCell<Vec<Box<dyn Show>>> {
    RefCell::new(vec![Box::new(K)])
}
fn main() {
    let m: Mutex<Box<dyn Fn() -> u8>> = Mutex::new(Box::new(|| 2));
    println!("{}", (m.lock().unwrap())());
    let h = Hooks { f: Mutex::new(Some(Box::new(|| 3))) };
    println!("{}", h.f.lock().unwrap().take().unwrap()());
    let r: Rc<RefCell<Option<Box<dyn Show>>>> = Rc::new(RefCell::new(Some(Box::new(K))));
    println!("{}", r.borrow().as_ref().unwrap().show());
    let c: Cell<fn() -> u8> = Cell::new(one);
    println!("{}", c.get()());
    let w: RwLock<&[u8]> = RwLock::new(&[1, 2, 3]);
    println!("{}", w.read().unwrap().len());
    let a: Arc<Option<Box<dyn Fn() -> u8 + Send + Sync>>> = Arc::new(Some(Box::new(|| 5)));
    println!("{}", a.as_ref().as_ref().unwrap()());
    let b: Box<Box<dyn Show>> = Box::new(Box::new(K));
    println!("{}", b.show());
    println!("{}", mk().borrow()[0].show());
}
