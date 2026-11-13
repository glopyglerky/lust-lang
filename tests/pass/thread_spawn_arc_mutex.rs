use std::thread;
use std::sync::{Arc, Mutex};
fn main() {
    let c = Arc::new(Mutex::new(0u64));
    let mut hs = Vec::new();
    for i in 0..4u64 {
        let c = Arc::clone(&c);
        hs.push(thread::spawn(move || { for _ in 0..1000 { *c.lock().unwrap() += 1; } i * 10 }));
    }
    let mut tot = 0;
    for h in hs { tot += h.join().unwrap(); }
    println!("{} {}", tot, *c.lock().unwrap());
}
