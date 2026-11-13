// A `&'static P` captured by ref in a closure passed to `any` inside
// `KEY.with`: the capture does not flow into the iterator (R18).
use std::cell::RefCell;
struct P { id: u32 }
static PROG: P = P { id: 1 };
thread_local! { static G: RefCell<Vec<&'static P>> = const { RefCell::new(Vec::new()) }; }
fn a(v: &'static P) -> bool { G.with(|g| g.borrow().iter().any(|&e| std::ptr::eq(e as *const P, v as *const P))) }
fn b(v: &'static P) -> bool { G.with(|g| g.borrow().iter().any(|e| std::ptr::eq(*e, v))) }
fn main() { G.with(|g| g.borrow_mut().push(&PROG)); println!("{} {} {}", a(&PROG), b(&PROG), PROG.id); }
