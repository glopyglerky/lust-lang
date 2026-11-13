// An `impl FnOnce`/`impl Fn` return value unsized into `Box<dyn ..>`, `Rc<dyn ..>`
// and `&dyn ..`, in plain and generic bodies; the generic definer `take_once`
// sits after its user, whose `+ Send` target needs its hidden type's auto traits
// (ts-rust gopanic.rs GoThread::start).
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn mk(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn start<'a, F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'a,
    T: 'a,
{
    let slot = Arc::new(Mutex::new(Some(f)));
    let b: Box<dyn FnOnce() -> T + Send + 'a> = Box::new(take_once(&slot));
    b()
}

fn take_once<F: FnOnce() -> T, T>(slot: &Arc<Mutex<Option<F>>>) -> impl FnOnce() -> T + use<F, T> {
    let slot = Arc::clone(slot);
    move || {
        let f = slot.lock().unwrap().take();
        f.expect("only once")()
    }
}

fn main() {
    let b: Box<dyn Fn(i32) -> i32> = Box::new(mk(1));
    let r: Rc<dyn Fn(i32) -> i32 + Send> = Rc::new(mk(2));
    let g = mk(3);
    let d: &dyn Fn(i32) -> i32 = &g;
    println!("{} {} {}", b(10), r(10), d(10));
    let s = String::from("moved");
    println!("{}", start(move || s.len()));
    println!("{:?}", start(|| vec![1u8, 2]));
}
