// R02: a dyn type written in a body (let type, `as` cast, closure param) has an inferred
// object lifetime in rustc, not `'static` and not a caller's. So a value built from `'static`
// parts may go to a `'static` use, and one built from a borrow may be returned as `+ 'a`.
use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};
trait P { fn v(&self) -> u32; }
struct S;
impl P for S { fn v(&self) -> u32 { 3 } }
struct W<'w>(&'w u32);
impl P for W<'_> { fn v(&self) -> u32 { *self.0 + 1 } }
type Memo = Rc<dyn Fn() -> Option<Rc<dyn P>>>;
fn memo() -> Memo {
    let value: OnceCell<Option<Rc<dyn P>>> = OnceCell::new();
    let m: Memo = Rc::new(move || value.get_or_init(|| Some(Rc::new(S) as Rc<dyn P>)).clone());
    m
}
fn keep(f: Box<dyn Fn() -> u32>) -> u32 { f() }
fn after(f: impl FnOnce() -> u32 + 'static) -> u32 { f() }
fn borrowed<'a>(r: &'a u32) -> Box<dyn P + 'a> {
    let b: Box<dyn P> = Box::new(W(r));
    b
}
fn cast() -> u32 {
    let b = Box::new(S) as Box<dyn P>;
    keep(Box::new(move || b.v()))
}
fn param() -> u32 {
    let b = (|y: Option<Box<dyn P>>| y)(Some(Box::new(S)));
    keep(Box::new(move || b.as_ref().map_or(0, |p| p.v())))
}
fn weak() -> u32 {
    let p: Arc<dyn P + Send + Sync> = Arc::new(S);
    let ps: Arc<Mutex<Vec<Weak<dyn P + Send + Sync>>>> = Arc::new(Mutex::new(Vec::new()));
    ps.lock().unwrap().push(Arc::downgrade(&p));
    let ps2 = ps.clone();
    after(move || {
        let v: Vec<Arc<dyn P + Send + Sync>> = ps2.lock().unwrap().iter().filter_map(|w| w.upgrade()).collect();
        v.iter().map(|x| x.v()).sum()
    })
}
struct C(u32);
impl C {
    fn push(&mut self, n: u32) -> Option<Box<dyn FnOnce(&mut C)>> {
        Some(Box::new(move |c: &mut C| c.0 += n))
    }
    fn scope(&mut self) -> Box<dyn FnOnce(&mut C)> {
        let mut cleanup: Option<Box<dyn FnOnce(&mut C)>> = None;
        cleanup = self.push(2);
        Box::new(move |c: &mut C| {
            if let Some(f) = cleanup {
                f(c);
            }
        })
    }
}
fn main() {
    println!("{}", memo()().map_or(0, |p| p.v()));
    let n = 5u32;
    println!("{} {} {} {}", borrowed(&n).v(), cast(), param(), weak());
    let mut c = C(1);
    let f = c.scope();
    f(&mut c);
    println!("{}", c.0);
}
