// A one-value wrapper constructor whose argument coerces to a raw pointer: the expected
// `Cell<*const u8>` makes `&x` a `*const u8` coercion site, as in rustc (tf-build: the
// coercion's target was an inference variable that was never resolved, so the C had a
// unit-typed temporary).
use std::cell::{Cell, RefCell};
use std::sync::Mutex;

struct Node {
    v: u8,
}

fn holder(p: *const Node) -> Box<*const Node> {
    Box::new(p)
}

fn main() {
    let x: u8 = 42;
    let y: u8 = 7;
    let c: Cell<*const u8> = Cell::new(&x);
    println!("{}", std::ptr::eq(c.get(), &x));
    c.set(&y);
    println!("{} {}", std::ptr::eq(c.get(), &x), c.get().is_null());
    let mut z = 3u8;
    let m: RefCell<*mut u8> = RefCell::new(&mut z);
    let r: RefCell<*const u8> = RefCell::new(&mut z);
    println!("{} {}", m.borrow().is_null(), r.borrow().is_null());
    let n = Node { v: 1 };
    let b: Box<*const Node> = Box::new(&n);
    let h = holder(&n);
    println!("{} {}", std::ptr::eq(*b, *h), n.v);
    let k: Mutex<*const Node> = Mutex::new(&n);
    println!("{}", std::ptr::eq(*k.lock().unwrap(), &n));
    let o: Option<Cell<*const u8>> = Some(Cell::new(&x));
    println!("{}", o.map(|c| c.get() as usize == &x as *const u8 as usize).unwrap_or(false));
}
