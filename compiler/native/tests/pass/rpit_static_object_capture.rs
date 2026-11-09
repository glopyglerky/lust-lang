// An `impl FnOnce() + '_` that captures `self` and an `Rc<dyn Fn() -> u32>`
// ('static object lifetime): the caller's borrow need not be 'static (R18).
use std::cell::Cell;
use std::rc::Rc;
struct Program { depth: Cell<u32>, now: Option<Rc<dyn Fn() -> u32>> }
impl Program {
    fn begin(&self) -> impl FnOnce() + '_ {
        let now = self.now.clone();
        self.depth.set(self.depth.get() + 1);
        move || {
            let Some(now) = now else { return };
            self.depth.set(self.depth.get() - now())
        }
    }
}
struct Handler<'a> { program: &'a Program }
impl Handler<'_> {
    fn sig(&self) -> u32 { let done = self.program.begin(); let n = self.program.depth.get(); done(); n }
}
fn main() { let p = Program { depth: Cell::new(0), now: Some(Rc::new(|| 1)) }; let h = Handler { program: &p }; println!("{} {}", h.sig(), p.depth.get()); }
