// `b.then(f)` returns what `f` returns: a closure that borrows `self`
// and returns an `Rc<dyn Fn()>` (whose object lifetime is 'static) does
// not make `self` 'static (ts-rust compilerhost.rs:653).
use std::rc::Rc;
struct H { p: Rc<String> }
impl H {
    fn f(&self) -> Option<Rc<dyn Fn() -> usize>> {
        true.then(|| { let t = self.p.clone(); Rc::new(move || t.len()) as Rc<dyn Fn() -> usize> })
    }
    fn g(&self, on: bool) -> Option<Box<dyn Fn() -> usize>> {
        on.then(|| -> Box<dyn Fn() -> usize> { let t = self.p.clone(); Box::new(move || t.len() + 1) })
    }
}
fn main() {
    let h = H { p: Rc::new(String::from("ab")) };
    println!("{} {} {}", (h.f().unwrap())(), (h.g(true).unwrap())(), h.g(false).is_none());
}
