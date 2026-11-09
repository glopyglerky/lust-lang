// feature: threads.sync
// form: mpsc channel plus an atomic counter, joined before printing
use std::sync::atomic::{AtomicUsize, Ordering};
static COUNT: AtomicUsize = AtomicUsize::new(0);
fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    let h = std::thread::spawn(move || { COUNT.fetch_add(2, Ordering::SeqCst); tx.send(40).unwrap(); });
    let v = rx.recv().unwrap();
    h.join().unwrap();
    println!("{} {}", v + 2, COUNT.load(Ordering::SeqCst));
}
