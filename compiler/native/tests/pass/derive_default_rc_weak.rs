// `#[derive(Default)]` over an `rc::Weak<T>` field (ts-rust's ParseTask
// links its task data by `Weak<RefCell<_>>`): alloc's `impl<T> Default
// for Weak<T>` is `Weak::new()`, also through a generic `T: Default`.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Data {
    n: u8,
}

#[derive(Default)]
struct Task {
    name: String,
    data: Weak<RefCell<Data>>,
    subs: Vec<Rc<RefCell<Task>>>,
}

fn d<T: Default>() -> T {
    T::default()
}

fn main() {
    let mut t = Task::default();
    println!("{:?} {} {}", t.name, t.data.upgrade().is_none(), t.subs.len());
    let rc = Rc::new(RefCell::new(Data { n: 7 }));
    t.data = Rc::downgrade(&rc);
    println!("{:?} {}", t.data.upgrade().map(|d| d.borrow().n), Rc::weak_count(&rc));
    let w: Weak<u32> = d();
    println!("{:?} {} {}", w.upgrade(), w.strong_count(), w.weak_count());
    t.subs.push(Rc::new(RefCell::new(Task::default())));
    println!("{}", t.subs[0].borrow().data.upgrade().is_none());
}
