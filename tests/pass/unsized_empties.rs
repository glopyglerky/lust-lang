use std::rc::Rc; use std::sync::Arc;
fn main() {
    let a: Rc<[u8]> = Rc::from(vec![]); let b: Box<str> = Box::from(""); let c: Arc<str> = Arc::from(String::new());
    let d: Rc<[()]> = Rc::from(vec![(); 3]); let e: Arc<[u128]> = Arc::from(vec![1u128 << 100, 2]);
    println!("{:?} {:?} {:?} {:?} {:?} {}", a, b, c, d, e, d.len());
}
