// A panic while a Mutex guard is held
// poisons the mutex; the next lock reports the poison and can
// recover the value.
use std::sync::{Arc, Mutex};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let m = Arc::new(Mutex::new(7i32));
    let in_thread = Arc::clone(&m);
    let handle = std::thread::spawn(move || {
        let mut g = in_thread.lock().unwrap();
        *g = 8;
        panic!("poison-boom");
    });
    println!("thread panicked={}", handle.join().is_err());
    match m.lock() {
        Ok(_) => println!("not poisoned"),
        Err(poisoned) => println!("poisoned, value={}", poisoned.into_inner()),
    }
}
