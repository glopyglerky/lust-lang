// panic: there is no such thing as a release load
use std::sync::atomic::{AtomicUsize, Ordering};
fn main() { let a = AtomicUsize::new(1); let o = if a.load(Ordering::SeqCst) == 1 { Ordering::Release } else { Ordering::SeqCst }; println!("before"); println!("{}", a.load(o)); }
