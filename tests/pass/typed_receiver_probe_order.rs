// Typed receivers in rustc's probe order: at the Rc<N> step a
// self: Rc<Self> method (by value) wins over self: &Rc<Self> (autoref),
// and an autoref match at Rc<N> wins over a &self method of N.
use std::rc::Rc;
trait A { fn f(&self) -> u8; }
trait B { fn f(self: &Rc<Self>) -> u8; }
struct N;
impl A for N { fn f(&self) -> u8 { 1 } }
impl B for N { fn f(self: &Rc<Self>) -> u8 { 2 } }
fn first() -> (u8, u8) {
    let n = Rc::new(N);
    (n.f(), N.f())
}

trait C { fn g(self: Rc<Self>) -> u8; }
trait D { fn g(self: &Rc<Self>) -> u8; }
struct M;
impl C for M { fn g(self: Rc<Self>) -> u8 { 1 } }
impl D for M { fn g(self: &Rc<Self>) -> u8 { 2 } }
fn main() {
    let m = Rc::new(M);
    println!("{:?} {}", first(), m.g());
}
