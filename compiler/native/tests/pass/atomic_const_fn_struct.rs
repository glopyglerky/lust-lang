use std::sync::atomic::{AtomicUsize, Ordering};
struct Counter { hits: AtomicUsize, misses: AtomicUsize }
impl Counter { const fn new() -> Self { Counter { hits: AtomicUsize::new(0), misses: AtomicUsize::new(0) } } fn hit(&self) { self.hits.fetch_add(1, Ordering::Relaxed); } }
static GLOBAL: Counter = Counter::new();
fn main() { GLOBAL.hit(); GLOBAL.hit(); let c = Counter::new(); c.misses.fetch_add(5, Ordering::SeqCst); println!("{} {}", GLOBAL.hits.load(Ordering::SeqCst), c.misses.into_inner()); }
