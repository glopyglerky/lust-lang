use std::sync::{Arc, Mutex};
use std::thread;
trait Counter: Send { fn add(&mut self, n: u64); fn get(&self) -> u64; }
struct Plain(u64);
impl Counter for Plain { fn add(&mut self, n: u64) { self.0 += n; } fn get(&self) -> u64 { self.0 } }
struct Wide(u128, String);
impl Counter for Wide { fn add(&mut self, n: u64) { self.0 += n as u128; self.1.push('+'); } fn get(&self) -> u64 { self.0 as u64 + self.1.len() as u64 } }
impl Drop for Wide { fn drop(&mut self) { println!("drop wide {}", self.1.len()); } }
fn mk(w: bool) -> Arc<Mutex<dyn Counter>> { if w { Arc::new(Mutex::new(Wide(0, String::new()))) } else { Arc::new(Mutex::new(Plain(0))) } }
fn main() {
    for w in [false, true] {
        let c = mk(w);
        let hs: Vec<_> = (0..4).map(|i| { let c = c.clone(); thread::spawn(move || { for _ in 0..10 { c.lock().unwrap().add(i); } }) }).collect();
        for h in hs { h.join().unwrap(); }
        println!("{}", c.lock().unwrap().get());
    }
    let b: Box<Mutex<dyn Counter>> = Box::new(Mutex::new(Plain(5)));
    b.lock().unwrap().add(1);
    println!("{}", b.lock().unwrap().get());
}
