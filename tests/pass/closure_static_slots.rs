// A closure's inferred return and argument types keep a `'static` object
// lifetime (`Box<dyn FnOnce()>` from a field), and `LocalKey::with` returns
// only what its closure returns (tsrust T13: gostd/local.rs, osvfs.rs).
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
trait Fs {
    fn n(&self) -> u8;
}
struct A;
impl Fs for A {
    fn n(&self) -> u8 {
        7
    }
}
struct St {
    idle: RefCell<VecDeque<(u8, Box<dyn FnOnce()>)>>,
    jobs: RefCell<HashMap<u64, Box<dyn FnOnce()>>>,
}
thread_local! {
    static LOCAL: St = St { idle: RefCell::new(VecDeque::new()), jobs: RefCell::new(HashMap::new()) };
}
fn run_idle() -> bool {
    let job = LOCAL.with(|l| l.idle.borrow_mut().pop_front());
    match job {
        Some((_, job)) => {
            job();
            true
        }
        None => false,
    }
}
fn run_job(id: u64) {
    let job = LOCAL.with(|l| l.jobs.borrow_mut().remove(&id));
    if let Some(job) = job {
        job();
    }
}
fn fs() -> Rc<dyn Fs> {
    thread_local! {
        static OS: Rc<dyn Fs> = Rc::new(A);
    }
    OS.with(|r| Rc::clone(r))
}
fn main() {
    LOCAL.with(|l| l.idle.borrow_mut().push_back((1, Box::new(|| println!("idle")))));
    LOCAL.with(|l| l.jobs.borrow_mut().insert(3, Box::new(|| println!("job"))));
    println!("{}", run_idle());
    run_job(3);
    println!("{}", fs().n());
}
