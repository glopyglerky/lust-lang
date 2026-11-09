// expect: L0306
// rustc E0597: the RPIT value holds the `&c` borrow, and it is called
// after `c` is dropped.
use std::cell::Cell;
use std::rc::Rc;
fn mk<'a>(p: &'a Cell<u32>, n: Rc<dyn Fn() -> u32>) -> impl FnOnce() -> u32 + 'a { move || { p.set(1); n() } }
fn main() {
    let r;
    { let c = Cell::new(0u32); r = mk(&c, Rc::new(|| 2)); }
    println!("{}", r());
}
