use std::sync::{Arc, Mutex, Condvar};
use std::thread;
fn main() {
    let st = Arc::new((Mutex::new(0), Condvar::new()));
    let s2 = st.clone();
    let h = thread::spawn(move || { *s2.0.lock().unwrap() += 1; s2.1.notify_all(); Arc::strong_count(&s2) });
    let c = h.join().unwrap();
    println!("{} {} {}", *st.0.lock().unwrap(), c, Arc::strong_count(&st));
}
