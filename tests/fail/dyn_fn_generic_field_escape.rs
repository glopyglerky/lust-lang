// expect: L0306
// rustc E0597: a struct field `Box<dyn Fn(T)>` with `T = &'x u8` (R11).
use std::cell::RefCell;
struct S<'v, T> {
    f: Box<dyn Fn(T) + 'v>,
}
fn mk<'v, T: 'v>(v: &'v RefCell<Vec<T>>) -> S<'v, T> {
    S { f: Box::new(move |t| v.borrow_mut().push(t)) }
}
fn main() {
    let v: RefCell<Vec<&u8>> = RefCell::new(Vec::new());
    let s = mk(&v);
    {
        let x = 5u8;
        (s.f)(&x);
    }
    println!("{}", v.borrow()[0]);
}
