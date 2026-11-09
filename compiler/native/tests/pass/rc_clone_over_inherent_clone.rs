// `clone` on an Rc or Arc (a value or a shared reference to one) is the pointer's Clone even
// when the pointee has an inherent `clone` (ts-rust Rc<ProjectCollection>). `(*p).clone()`
// and `p.as_ref().clone()` call the inherent one.
use std::rc::Rc;
use std::sync::Arc;
struct PC { n: Vec<u32> }
impl PC {
    pub fn clone(&self) -> PC { PC { n: vec![9; self.n.len() + 1] } }
}
struct B { pc: Rc<PC>, a: Arc<PC> }
impl B {
    fn take(&self) -> Rc<PC> { self.pc.clone() }
}
fn main() {
    let b = B { pc: Rc::new(PC { n: vec![1, 2] }), a: Arc::new(PC { n: vec![3] }) };
    let r = b.pc.clone();
    let t = b.take();
    let a = b.a.clone();
    let deep = (*b.pc).clone();
    let viaref = b.pc.as_ref().clone();
    let rr = &b.pc;
    let c = rr.clone();
    println!("{} {} {} {} {}", Rc::strong_count(&b.pc), Arc::strong_count(&b.a), r.n.len(), t.n.len(), a.n.len());
    println!("{} {} {}", deep.n.len(), viaref.n.len(), c.n.len());
}
