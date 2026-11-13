use std::sync::{Arc, Barrier, Mutex};
use std::thread;
fn main() {
    let n = 5;
    let b = Arc::new(Barrier::new(n));
    let log = Arc::new(Mutex::new(Vec::new()));
    let hs: Vec<_> = (0..n).map(|i| { let b = b.clone(); let log = log.clone(); thread::spawn(move || { log.lock().unwrap().push(i); let r = b.wait(); (log.lock().unwrap().len(), r.is_leader()) }) }).collect();
    let rs: Vec<(usize, bool)> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    let leaders = rs.iter().filter(|r| r.1).count();
    println!("{} {}", leaders, rs.iter().all(|r| r.0 == n));
    let one = Barrier::new(1);
    let r = one.wait();
    println!("{:?} {:?}", r, one);
}
