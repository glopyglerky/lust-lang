// A captured `&Rc<T>` (or `Rc<T>`) passed where `&T` is expected: the
// overloaded deref borrows the pointer (ts-rust file_loader `import_names`,
// legacy_decorators `has_internal_static_reference`).
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
struct P { n: u32 }
struct Names(u32);
impl Names { fn of(p: &P) -> Names { Names(p.n * 2) } }
thread_local! { static KEPT: RefCell<HashMap<usize, Rc<Names>>> = RefCell::new(HashMap::new()); }
fn names(file: &Rc<P>) -> Rc<Names> {
    let key = Rc::as_ptr(file) as usize;
    KEPT.with(|kept| {
        if let Some(n) = kept.borrow().get(&key) { return n.clone(); }
        let n = Rc::new(Names::of(file));
        kept.borrow_mut().insert(key, n.clone());
        n
    })
}
trait R { fn r(&self) -> u32; }
struct Q;
impl R for Q { fn r(&self) -> u32 { 4 } }
struct T { ec: Rc<P>, res: Box<dyn R> }
fn uses(ec: &P, r: &dyn R, k: u32) -> bool { ec.n + r.r() == k }
impl T {
    fn has(&self) -> bool {
        let ec = &self.ec;
        let resolver = &*self.res;
        [5u32, 9].iter().any(|&k| uses(ec, resolver, k))
    }
}
fn main() {
    let f = Rc::new(P { n: 5 });
    println!("{} {}", names(&f).0, names(&f).0);
    let t = T { ec: Rc::new(P { n: 5 }), res: Box::new(Q) };
    println!("{}", t.has());
    let s = String::from("ab"); let rs = &s;
    let g = |x: &str| x == rs.as_str() && takes(rs);
    println!("{}", g("ab"));
}
fn takes(s: &str) -> bool { s.len() == 2 }
