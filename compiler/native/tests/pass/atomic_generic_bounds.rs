use std::sync::atomic::{AtomicUsize, Ordering};
fn shared<T: Send + Sync + std::fmt::Debug>(t: &T) { println!("{:?}", t); }
fn d<T: Default + std::fmt::Debug>() -> T { T::default() }
fn main() { let a = AtomicUsize::new(4); shared(&a); let b: AtomicUsize = d(); b.fetch_add(1, Ordering::SeqCst); println!("{:?}", b); }
