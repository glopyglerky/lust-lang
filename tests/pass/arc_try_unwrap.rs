use std::sync::Arc;
fn main() {
    let a = Arc::new(vec![1u8]);
    let w = Arc::downgrade(&a);
    let r = Arc::try_unwrap(a);
    println!("{:?} {}", r, w.upgrade().is_none());
    let b = Arc::new(3);
    let b2 = b.clone();
    match Arc::try_unwrap(b) { Ok(v) => println!("ok {}", v), Err(e) => println!("err {} {}", e, Arc::strong_count(&e)) }
    println!("{:?}", Arc::into_inner(b2));
    let c = Arc::new(String::from("c"));
    let c2 = Arc::clone(&c);
    println!("{:?} {:?}", Arc::into_inner(c), Arc::into_inner(c2));
    let d = Arc::new(String::from("d"));
    let d2 = d.clone();
    let s1 = Arc::unwrap_or_clone(d);
    let s2 = Arc::unwrap_or_clone(d2);
    println!("{} {}", s1, s2);
}
