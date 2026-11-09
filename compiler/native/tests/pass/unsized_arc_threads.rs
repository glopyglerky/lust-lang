use std::sync::Arc;
use std::thread;
fn main() {
    let s: Arc<str> = Arc::from("shared");
    let mut hs = Vec::new();
    for i in 0..4 { let s = s.clone(); hs.push(thread::spawn(move || s.len() + i)); }
    let mut t = 0; for h in hs { t += h.join().unwrap(); }
    println!("{t} {}", Arc::strong_count(&s));
}
