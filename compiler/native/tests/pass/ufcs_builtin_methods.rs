// Builtin methods by path: the first argument is the receiver, coerced
// to `&Self` / `&mut Self` (through references and smart pointers), and
// a few as fn values.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Mutex;

fn start<H>(mut clone: impl FnMut(std::thread::Builder, fn() -> i32) -> std::io::Result<H>) -> H {
    clone(std::thread::Builder::new().name("w".into()), seven).unwrap()
}
fn seven() -> i32 {
    7
}

fn main() {
    let mut v = vec![1, 2];
    Vec::push(&mut v, 3);
    let b = Box::new(vec![4]);
    let r = Rc::new(String::from("abc"));
    println!("{} {} {} {}", Vec::len(&v), Vec::len(&&v), Vec::len(&b), String::len(&r));
    println!("{} {:?}", Vec::is_empty(&mut v), Vec::pop(&mut v));
    let c = Cell::new(3);
    Cell::set(&c, 4);
    let rc = RefCell::new(vec![1]);
    RefCell::borrow_mut(&rc).push(2);
    println!("{} {:?}", Cell::get(&c), RefCell::borrow(&rc));
    let m = Mutex::new(1);
    *Mutex::lock(&m).unwrap() += 1;
    println!("{:?}", Mutex::into_inner(m));
    let h = std::thread::Builder::spawn(std::thread::Builder::new(), || 5).unwrap();
    println!("{}", h.join().unwrap());
    let w = vec![vec![1, 2], vec![]];
    let lens: Vec<usize> = w.iter().map(Vec::len).collect();
    let empty: Vec<bool> = w.iter().map(Vec::is_empty).collect();
    println!("{:?} {:?} {}", lens, empty, w.first().map_or(0, Vec::len));
    let t = start(std::thread::Builder::spawn);
    println!("{}", t.join().unwrap());
}
