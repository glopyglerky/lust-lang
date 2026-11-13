// A closure stores a captured `'static` value into a `'static` slot of its
// argument (the bound's `Vec<&'static str>`, a `Box<dyn Fn()>` behind a
// `RefCell`): the body needs the capture to be `'static`, and the creator
// proves it (gostd/context.rs `close`).
use std::cell::RefCell;
type B = Box<dyn FnOnce() -> u8 + Send>;
fn with<R>(f: impl FnOnce(&mut Vec<&'static str>) -> R) -> R {
    let mut v = Vec::new();
    let r = f(&mut v);
    println!("{:?}", v);
    r
}
fn with_cell<R>(f: impl FnOnce(&RefCell<Option<Vec<B>>>) -> R) -> R {
    let c = RefCell::new(Some(Vec::new()));
    let r = f(&c);
    for g in c.into_inner().unwrap() {
        println!("{}", g());
    }
    r
}
fn main() {
    let s: &'static str = "a";
    with(|d| d.push(s));
    let k = 7u8;
    let ws: Vec<B> = vec![Box::new(move || k)];
    let left = with_cell(|d| match &mut *d.borrow_mut() {
        Some(v) => {
            v.extend(ws);
            0
        }
        None => 1,
    });
    println!("{left}");
}
