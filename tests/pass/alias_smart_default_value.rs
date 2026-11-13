// `Alias::default` for a type alias of Rc, Arc, or Box, as a call and as a
// fn value (ts-rust `.then(DeferredWrites::default)`): std's
// `impl<T: Default> Default` for each pointer builds a new default value.
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Default, Debug)]
struct Pending {
    writes: Vec<String>,
}

type DeferredWrites = Arc<Mutex<Pending>>;
type Cached = Rc<RefCell<Vec<u8>>>;
type Boxed = Box<Vec<u16>>;
type Text = Rc<str>;
type Shared<T> = Rc<RefCell<T>>;

fn main() {
    let d = true.then(DeferredWrites::default);
    d.as_ref().unwrap().lock().unwrap().writes.push("a".to_string());
    println!("{:?}", d);
    let c = Cached::default();
    c.borrow_mut().push(7);
    println!("{:?} {}", c, Rc::strong_count(&c));
    let mut o: Option<Boxed> = None;
    o.get_or_insert_with(Boxed::default).push(3);
    println!("{:?}", o);
    let f = Text::default;
    println!("[{}]", f());
    let s: Shared<String> = Shared::default();
    s.borrow_mut().push_str("x");
    let n = false.then(Shared::<u8>::default);
    println!("{:?} {:?}", s, n);
}
