// &Rc<T> coerces to &T, to &str, and to a slice.
use std::rc::Rc;
fn show(x: &i32) { println!("{}", x); }
fn shout(s: &str) { println!("{}", s); }
fn main() {
    let a = Rc::new(4);
    show(&a);
    let s = Rc::new("hey".to_string());
    shout(&s);
    let v = Rc::new(vec![8, 9]);
    let sl: &[i32] = &v;
    println!("{:?}", sl);
}
