// A closure is Clone when its captures are (ts-rust session.rs
// after_delay_or_done clones closures over an Rc and an Arc<Mutex>):
// rustc's clone shim clones each capture in order, unless the closure
// is Copy.
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Loud(u8);

impl Clone for Loud {
    fn clone(&self) -> Loud {
        println!("clone {}", self.0);
        Loud(self.0)
    }
}

fn register<F: Fn() + Clone + 'static>(f: F) -> Option<u8> {
    let g = f.clone();
    g();
    f();
    None
}

fn main() {
    let rest: Rc<Cell<Option<Box<dyn FnOnce()>>>> = Rc::new(Cell::new(Some(Box::new(|| println!("rest")))));
    let run = move || {
        if let Some(r) = rest.take() {
            r();
        }
    };
    let again = run.clone();
    again();
    run();

    let early = Arc::new(Mutex::new(Some(5u8)));
    let release = move || drop(early.lock().unwrap_or_else(|e| e.into_inner()).take());
    if register(release.clone()).is_none() {
        release();
    }

    let name = String::from("n");
    let (a, b) = (Loud(7), Loud(8));
    let show = move |x: u8| format!("{name}{:?}{x}{:?}", b, a);
    let show2 = show.clone();
    println!("{} {}", show(1), show2(2));

    // A closure over shared borrows is Copy.
    let v = vec![1, 2];
    let sum = || v.iter().sum::<i32>();
    let sum2 = sum.clone();
    println!("{} {}", sum(), sum2());
}
