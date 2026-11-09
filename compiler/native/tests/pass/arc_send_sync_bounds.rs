use std::sync::Arc;
fn need<T: Send + Sync + 'static>(t: T) -> T { t }
fn main() { let a = need(Arc::new(vec![String::from("x")])); println!("{:?}", a); let w = need(Arc::downgrade(&a)); println!("{}", w.upgrade().is_some()); }
