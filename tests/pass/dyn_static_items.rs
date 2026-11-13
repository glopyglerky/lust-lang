// `dyn` types in statics, consts, and `thread_local!` items whose values are
// built at run time (no vtable at compile time).
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};
trait Tr: Send + Sync {
    fn v(&self) -> u32;
}
struct A(u32);
impl Tr for A {
    fn v(&self) -> u32 {
        self.0
    }
}
trait Logger {
    fn log(&self) -> u32;
}
struct Q;
impl Logger for Q {
    fn log(&self) -> u32 {
        9
    }
}
static L: LazyLock<Box<dyn Tr>> = LazyLock::new(|| Box::new(A(3)));
static M: Mutex<Vec<Box<dyn Fn() -> u32 + Send>>> = Mutex::new(Vec::new());
static V: Mutex<Option<Box<dyn Tr>>> = Mutex::new(None);
const NOP: Option<Rc<dyn Logger>> = None;
const NONE: Option<&dyn Tr> = None;
thread_local! {
    static AFTER: RefCell<Option<Rc<dyn Fn() -> u32>>> = const { RefCell::new(None) };
}
fn main() {
    println!("{}", L.v());
    M.lock().unwrap().push(Box::new(|| 4));
    M.lock().unwrap().push(Box::new(|| 5));
    let s: u32 = M.lock().unwrap().iter().map(|f| f()).sum();
    println!("{}", s);
    *V.lock().unwrap() = Some(Box::new(A(7)));
    println!("{}", V.lock().unwrap().as_ref().map_or(0, |b| b.v()));
    let mut x = NOP;
    println!("{} {}", x.is_none(), NONE.is_none());
    x = Some(Rc::new(Q));
    println!("{}", x.unwrap().log());
    AFTER.set(Some(Rc::new(|| 11)));
    println!("{}", AFTER.take().map_or(0, |f| f()));
    println!("{}", AFTER.take().is_none());
    M.lock().unwrap().clear();
}
