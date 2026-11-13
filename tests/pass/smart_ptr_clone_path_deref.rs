// `Rc::clone`, `Arc::clone`, `Box::clone`, and `sync::Weak::clone` by
// path deref their argument to the pointer (`&&rc` to `&Rc`).
use std::rc::Rc;
use std::sync::{Arc, Weak};

fn main() {
    let a = Rc::new(vec![1]);
    let refs = vec![&a];
    let r: Vec<Rc<Vec<i32>>> = refs.iter().map(|x| Rc::clone(x)).collect();
    let b = Arc::new(2);
    let x = &&b;
    let c: Arc<i32> = Arc::clone(x);
    let d = Box::new(3);
    let e: Box<i32> = Box::clone(&&d);
    let w = Arc::downgrade(&b);
    let z: Weak<i32> = Weak::clone(&&w);
    println!("{:?} {} {} {} {:?} {}", r, Rc::strong_count(&a), c, e, z.upgrade(), Arc::strong_count(&b));
}
