// feature: threads.sync
// form: Arc<Mutex<T>> shared with a spawned thread, joined for the result
use std::sync::{Arc, Mutex};
fn main() {
    let cell = Arc::new(Mutex::new(0i32));
    let c2 = Arc::clone(&cell);
    let h = std::thread::spawn(move || { *c2.lock().unwrap() += 5; });
    h.join().unwrap();
    println!("{}", *cell.lock().unwrap());
}
