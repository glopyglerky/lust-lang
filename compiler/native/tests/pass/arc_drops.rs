use std::sync::{Arc, Weak};
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let a = Arc::new(D("a"));
    let w = Arc::downgrade(&a);
    {
        let b = a.clone();
        println!("inner {} {}", Arc::strong_count(&b), Arc::weak_count(&b));
    }
    println!("after inner");
    let w2 = w.clone();
    drop(a);
    println!("after a: {} {} {}", w.strong_count(), w.weak_count(), w2.upgrade().is_none());
    drop(w);
    println!("end");
    let x = Arc::new(D("x"));
    let y = Arc::new(D("y"));
    let v = vec![x.clone(), y.clone(), x];
    println!("{}", Arc::strong_count(&y));
    drop(v);
    println!("vec gone");
}
