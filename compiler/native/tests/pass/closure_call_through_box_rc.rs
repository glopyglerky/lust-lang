// A closure called through a Box or an Rc (the callee derefs, as rustc's
// `impl Fn for Box<F>` and callee autoderef do); an FnMut or FnOnce one
// in a Box is called through the pointee, `(*m)()`.
use std::rc::Rc;

fn main() {
    let k = 5u32;
    let b = Box::new(move |x: u32| x + k);
    let r = Rc::new(|x: u32| x * 2);
    let r2 = r.clone();
    let mut n = 0u32;
    let mut m = Box::new(|| {
        n += 1;
        n
    });
    (*m)();
    let last = (*m)();
    let s = String::from("once");
    let o = Box::new(move || s);
    println!("{} {} {} {} {} {}", b(1), (*b)(2), r(3), r2(4), last, (*o)());
}
