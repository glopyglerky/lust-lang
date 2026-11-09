// expect: L0306
// rustc E0597: the visitor's `C = &mut u32` is a free region of its
// `dyn Fn(.., &mut V<'a, C>)` field, so it must outlive the visitor (R11).
use std::rc::Rc;
type VisitFn<'a, C> = Rc<dyn Fn(u32, &mut V<'a, C>) -> u32 + 'a>;
struct V<'a, C> {
    visit: Option<VisitFn<'a, C>>,
    ctx: C,
}
impl<'a, C> V<'a, C> {
    fn call(&mut self, n: u32) -> u32 {
        let f = self.visit.clone().unwrap();
        f(n, self)
    }
}
fn main() {
    let mut v;
    {
        let mut c = 0u32;
        v = V { visit: None, ctx: &mut c };
    }
    v.visit = Some(Rc::new(|n, v: &mut V<'_, &mut u32>| {
        *v.ctx += 1;
        n
    }));
    println!("{}", v.call(1));
}
