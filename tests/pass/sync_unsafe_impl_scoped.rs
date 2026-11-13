use std::cell::Cell;
struct C { c: Cell<u8> }
unsafe impl Sync for C {}
fn need<T: Send + Sync>(_: &T) {}
fn main() { let s = C { c: Cell::new(3) }; need(&s); let r = &s; std::thread::scope(|sc| { sc.spawn(move || println!("{}", r.c.get())); }); }
