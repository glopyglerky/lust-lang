// expect: L0641
// Valid Rust: `try_borrow` on a `RefCell<dyn T>`; lust builds only `borrow` and `borrow_mut` for a dyn cell (w-unsized).
use std::cell::RefCell; use std::rc::Rc;
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn main() { let a: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(S)); println!("{}", a.try_borrow().is_ok()); }
