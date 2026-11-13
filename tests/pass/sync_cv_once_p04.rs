use std::sync::{Mutex, Condvar, Barrier};
static M: Mutex<u32> = Mutex::new(0);
static CV: Condvar = Condvar::new();
static B: Barrier = Barrier::new(2);
fn main() {
    let h = std::thread::spawn(|| { B.wait(); *M.lock().unwrap() = 5; CV.notify_all(); });
    B.wait();
    let g = CV.wait_while(M.lock().unwrap(), |v| *v == 0).unwrap();
    println!("{}", *g);
    drop(g);
    h.join().unwrap();
}
