// An RPIT closure that captures `&'a Cell` and a `'static` dyn object,
// then is called: `&r` holds `r`'s captures for its own region only, so
// the dyn's `'static` does not make the `&c` loan `'static`.
use std::cell::Cell;
use std::rc::Rc;
fn mk<'a>(p: &'a Cell<u32>, n: Rc<dyn Fn() -> u32>) -> impl FnOnce() -> u32 + 'a { move || { p.set(p.get() + 1); n() } }
fn mk_box<'a>(p: &'a Cell<u32>, n: Box<dyn Fn() -> u32>) -> impl Fn() -> u32 + 'a { move || { p.set(p.get() + 10); n() } }
fn run() -> u32 {
    let c = Cell::new(0u32);
    let r = mk(&c, Rc::new(|| 2));
    let a = r();
    let b = mk_box(&c, Box::new(|| 3));
    let s = a + b() + b();
    s + c.get()
}
fn main() { println!("{}", run()); }
