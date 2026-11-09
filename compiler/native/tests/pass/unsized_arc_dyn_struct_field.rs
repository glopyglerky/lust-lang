use std::sync::{Arc, Mutex};
trait Clock { fn now(&self) -> u64; }
struct Fixed(u64);
impl Clock for Fixed { fn now(&self) -> u64 { self.0 } }
struct Sys { clock: Arc<dyn Clock + Send + Sync>, hooks: Mutex<Vec<Arc<dyn Fn() -> u64 + Send + Sync>>> }
fn main() {
    let s = Sys { clock: Arc::new(Fixed(42)), hooks: Mutex::new(Vec::new()) };
    let c = s.clock.clone();
    s.hooks.lock().unwrap().push(Arc::new(move || c.now() + 1));
    let hs = s.hooks.lock().unwrap().clone();
    for h in hs.iter() { println!("{}", h()); }
    println!("{}", s.clock.now());
}
