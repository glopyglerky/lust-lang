// Rc<T> is Clone with no bound on T; boxes in generic fns.
use std::rc::Rc;
#[derive(Debug)]
struct NoClone { n: i32 }
fn share<T>(x: Rc<T>) -> (Rc<T>, Rc<T>) { let y = x.clone(); (x, y) }
fn main() {
    let (a, b) = share(Rc::new(NoClone { n: 3 }));
    println!("{:?} {:?} {}", a, b, Rc::strong_count(&a));
    let l = vec![a.clone(), b.clone()];
    println!("{}", l.len());
}
