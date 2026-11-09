use std::sync::Arc; use std::thread;
trait Job: Send + Sync { fn run(&self, x: u32) -> u32; }
struct Add(u32);
impl Job for Add { fn run(&self, x: u32) -> u32 { x + self.0 } }
fn mk() -> Arc<dyn Job> { Arc::new(Add(5)) }
fn main() {
    let j = mk();
    let hs: Vec<_> = (0..3).map(|i| { let j = j.clone(); thread::spawn(move || j.run(i)) }).collect();
    let s: u32 = hs.into_iter().map(|h| h.join().unwrap()).sum();
    println!("{s} {}", Arc::strong_count(&j));
}
