// A static `LazyLock` forced from 8 threads runs its init once (W-STD-C
// LazyLock over W-THREADS's OnceLock).
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
static INITS: AtomicUsize = AtomicUsize::new(0);
static TABLE: LazyLock<Vec<u32>> = LazyLock::new(|| { INITS.fetch_add(1, Ordering::SeqCst); (0..1000).collect() });
fn main() {
    let hs: Vec<_> = (0..8).map(|i| std::thread::spawn(move || TABLE[i * 10] + TABLE.len() as u32)).collect();
    let s: u32 = hs.into_iter().map(|h| h.join().unwrap()).sum();
    println!("{} {}", s, INITS.load(Ordering::SeqCst));
}
