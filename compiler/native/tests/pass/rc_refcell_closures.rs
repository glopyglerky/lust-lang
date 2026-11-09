// Rc<RefCell<T>> fixture set: boxed closures that share state, Cell counters, and a snapshot clone.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn main() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let count = Rc::new(Cell::new(0u32));
    let mut handlers: Vec<Box<dyn Fn(&str)>> = Vec::new();
    for tag in ["x", "y"] {
        let log = Rc::clone(&log);
        let count = Rc::clone(&count);
        handlers.push(Box::new(move |msg: &str| {
            count.set(count.get() + 1);
            log.borrow_mut().push(format!("{}:{}", tag, msg));
        }));
    }
    for h in &handlers {
        h("hi");
    }
    handlers[1]("bye");
    println!("{:?} {} {}", log.borrow(), count.get(), Rc::strong_count(&log));
    drop(handlers);
    println!("{}", Rc::strong_count(&log));
    let acc = Rc::new(RefCell::new(String::new()));
    let push = {
        let acc = acc.clone();
        move |c: char| acc.borrow_mut().push(c)
    };
    "abc".chars().for_each(push);
    println!("{} {}", acc.borrow(), acc.borrow().len());
    let shared = Rc::new(RefCell::new(vec![3, 1, 2]));
    shared.borrow_mut().sort();
    let snapshot = shared.borrow().clone();
    shared.borrow_mut().push(4);
    println!("{:?} {:?}", snapshot, shared.borrow());
}
