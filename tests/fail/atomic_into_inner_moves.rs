// expect: L0301
use std::sync::atomic::{AtomicU16, Ordering};
fn main() { let a = AtomicU16::new(65535); a.fetch_add(2, Ordering::Relaxed); let v = a.into_inner(); println!("{}", v); println!("{:?}", a); }
