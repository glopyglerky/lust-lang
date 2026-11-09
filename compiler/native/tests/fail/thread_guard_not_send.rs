// expect: L0612
use std::thread;
use std::sync::Mutex;
static M: Mutex<u8> = Mutex::new(0);
fn main() { let g = M.lock().unwrap(); thread::spawn(move || { let x = g; }).join().unwrap(); }
