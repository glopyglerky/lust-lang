use std::rc::Rc; use std::sync::Arc;
fn main() { let r: Rc<str> = Rc::from("w"); println!("{} {}", Rc::strong_count(&r), Rc::weak_count(&r)); let a: Arc<[u8]> = Arc::from(vec![1]); let a2 = a.clone(); println!("{} {} {}", Arc::strong_count(&a), Arc::weak_count(&a), Arc::ptr_eq(&a, &a2)); }
