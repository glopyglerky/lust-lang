use std::rc::Rc; use std::sync::Arc;
#[derive(Default, Debug)]
struct D { a: Box<str>, b: Rc<str>, c: Arc<str>, d: Rc<[u8]>, e: Arc<[i32]> }
fn main() {
    let d = D::default();
    println!("{:?}", d);
    let x: Arc<str> = Default::default(); let y: Arc<str> = Default::default();
    println!("{} {}", Arc::ptr_eq(&x, &y), Arc::strong_count(&x) >= 2);
}
