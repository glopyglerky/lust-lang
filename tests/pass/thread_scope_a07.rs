use std::thread;
use std::sync::atomic::{AtomicU32, Ordering};
fn main() {
    let n = AtomicU32::new(0);
    let words = ["a", "bb", "ccc"];
    thread::scope(|s| {
        for w in &words { let n = &n; s.spawn(move || { n.fetch_add(w.len() as u32, Ordering::SeqCst); }); }
    });
    println!("{}", n.load(Ordering::SeqCst));
}
