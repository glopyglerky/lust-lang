use std::sync::{Arc, Mutex, Condvar};
use std::thread;
fn main() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let p2 = Arc::clone(&pair);
    let h = thread::spawn(move || { *p2.0.lock().unwrap() = true; p2.1.notify_one(); });
    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started { started = cvar.wait(started).unwrap(); }
    println!("started {}", *started);
    drop(started);
    h.join().unwrap();
    let q = Arc::new((Mutex::new(Vec::<u32>::new()), Condvar::new()));
    let q2 = q.clone();
    let prod = thread::spawn(move || { for i in 0..5 { q2.0.lock().unwrap().push(i); q2.1.notify_all(); } });
    let g = q.1.wait_while(q.0.lock().unwrap(), |v| v.len() < 5).unwrap();
    println!("{:?}", *g);
    drop(g);
    prod.join().unwrap();
    println!("{:?}", Condvar::new());
}
