// An Fn-bound type param unsizes to `Box<dyn Fn>`, `Rc<dyn Fn>`, and
// `&dyn Fn` (the `hooks.push(Box::new(f))` shape); each instance builds
// the vtable of its closure.
use std::rc::Rc;
struct Bus { hooks: Vec<Box<dyn Fn(u32) -> u32>>, shared: Option<Rc<dyn Fn() -> String>>, mut_hooks: Vec<Box<dyn FnMut(u32)>> }
impl Bus {
    fn on<F: Fn(u32) -> u32 + 'static>(&mut self, f: F) { self.hooks.push(Box::new(f)); }
    fn share<F>(&mut self, f: F) where F: Fn() -> String + 'static { self.shared = Some(Rc::new(f)); }
    fn on_mut(&mut self, f: impl FnMut(u32) + 'static) { self.mut_hooks.push(Box::new(f)); }
    fn fire(&mut self, x: u32) -> u32 { let mut a = x; for h in &self.hooks { a = h(a); } for m in self.mut_hooks.iter_mut() { m(a); } a }
}
fn apply_ref<F: Fn(u8) -> u8>(f: &F) -> u8 { let d: &dyn Fn(u8) -> u8 = f; d(2) }
fn main() {
    let mut b = Bus { hooks: Vec::new(), shared: None, mut_hooks: Vec::new() };
    let k = 4;
    b.on(move |x| x + k);
    b.on(|x| x * 3);
    b.share(|| String::from("s"));
    b.on_mut(|x| println!("saw {}", x));
    println!("{}", b.fire(1));
    if let Some(s) = &b.shared { println!("{}", s()); }
    println!("{}", apply_ref(&|v| v + 40));
}
