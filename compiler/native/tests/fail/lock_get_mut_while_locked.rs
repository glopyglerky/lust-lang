// expect: L0303
use std::sync::Mutex;
fn main() { let mut m = Mutex::new(5); let g = m.lock().unwrap(); let r = m.get_mut().unwrap(); *r = 1; println!("{}", *g); }
