use std::rc::Rc; use std::sync::Arc;
fn main() {
    let inner: Vec<Rc<str>> = vec![Rc::from("x"), Rc::from("y")];
    let outer: Rc<[Rc<str>]> = Rc::from(inner);
    let o2 = outer.clone();
    println!("{:?} {}", o2, Rc::strong_count(&outer[0]));
    let opt: Option<Arc<[u8]>> = Some(Arc::from(&b"hi"[..]));
    println!("{:?}", opt.as_ref().map(|a| &a[..]));
    let boxes: Vec<Box<str>> = vec!["a".into(), "b".into()];
    println!("{:?}", boxes);
}
