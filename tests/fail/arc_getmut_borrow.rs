// expect: L0303
use std::sync::Arc;
fn main() { let mut a = Arc::new(1); let r = Arc::get_mut(&mut a).unwrap(); let b = a.clone(); *r += 1; println!("{} {}", a, b); }
