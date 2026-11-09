use std::sync::{Arc, Weak};
#[derive(Debug, Clone, PartialEq, Default)]
struct P { x: i32, name: String }
fn main() {
    let a = Arc::new(P { x: 3, name: "a".to_string() });
    let b = Arc::clone(&a);
    let c = a.clone();
    println!("{} {} {:?}", Arc::strong_count(&a), a.x, b);
    let w: Weak<P> = Arc::downgrade(&a);
    println!("{} {}", Arc::weak_count(&a), w.strong_count());
    drop(b); drop(c);
    match w.upgrade() { Some(p) => println!("{}", p.name), None => println!("none") }
    let mut a = a;
    Arc::make_mut(&mut a).x += 10;
    println!("{:?} {} {}", a, Arc::strong_count(&a), w.upgrade().is_none());
    let d = Arc::new(5u64);
    let e = d.clone();
    println!("{:?} {}", Arc::try_unwrap(d), Arc::ptr_eq(&e, &e));
    println!("{:?} {:?}", Arc::into_inner(e), Weak::<i32>::new().upgrade());
    let mut f = Arc::new(vec![1, 2]);
    Arc::get_mut(&mut f).unwrap().push(3);
    println!("{:?} {}", f, f.len());
    let g: Arc<P> = Default::default();
    println!("{:?} {}", g, *g == P::default());
    let h = Arc::new(String::from("hey"));
    let s: &str = &h;
    println!("{} {}", s, h.to_uppercase());
    println!("{}", Arc::unwrap_or_clone(Arc::new(7)));
}
