// thread_local cells in release C are native (__thread) cells, one per static: accesses
// from several fns and from a closure reach the same cell, each thread has its own, try_with
// sees it, a RefCell value is borrowed through it, and a value with a Drop drops once, when
// its thread ends. (lust takes key methods only on a static by name, L0640, so every
// access names its static.)
use std::cell::{Cell, RefCell};

struct Noisy(&'static str, u32);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {} {}", self.0, self.1);
    }
}

thread_local! {
    static COUNT: Cell<u32> = const { Cell::new(0) };
    static NAME: RefCell<Noisy> = RefCell::new(Noisy("init", 0));
}

#[inline(never)]
fn bump() -> u32 {
    COUNT.set(COUNT.get() + 1);
    COUNT.get()
}

#[inline(never)]
fn peek() -> Option<u32> {
    COUNT.try_with(|c| c.get()).ok()
}

fn count(tag: &str, start: u32) -> u32 {
    COUNT.set(start);
    let a = bump();
    let add = |n: u32| COUNT.set(COUNT.get() + n);
    add(5);
    let b = bump();
    println!("{tag}: {a} {b} {} {:?}", COUNT.get(), peek());
    b
}

fn main() {
    let h = std::thread::spawn(|| {
        let b = count("thread", 10);
        NAME.with(|v| {
            let mut v = v.borrow_mut();
            v.0 = "thread";
            v.1 = b;
        });
        println!("thread name: {} {}", NAME.with(|v| v.borrow().0), NAME.with(|v| v.borrow().1));
        b
    });
    let r = h.join().unwrap();
    let m = count("main", 100);
    println!("joined {r}, main {m}, main count still {}", COUNT.get());
}
