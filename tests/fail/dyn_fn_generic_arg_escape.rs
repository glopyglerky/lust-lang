// expect: L0306
// rustc E0597: `Box<dyn Fn(T)>` with `T = &'x u8` takes only `'x`, not any
// region: the closure keeps its argument in `v` (R11).
use std::cell::RefCell;
fn mk<'v, T: 'v>(v: &'v RefCell<Vec<T>>) -> Box<dyn Fn(T) + 'v> {
    Box::new(move |t| v.borrow_mut().push(t))
}
fn main() {
    let v: RefCell<Vec<&u8>> = RefCell::new(Vec::new());
    let f = mk(&v);
    {
        let x = 5u8;
        f(&x);
    }
    println!("{}", v.borrow()[0]);
}
