// expect: L0733
// rustc: lifetime may not live long enough (an `Rc<dyn Fn>` field is `'static`).
use std::rc::Rc;
struct H { f: Rc<dyn Fn() -> usize> }
fn mk(s: &str) -> H { H { f: Rc::new(move || s.len()) } }
fn main() { let h = mk("abc"); println!("{}", (h.f)()); }
