// expect: L0303
use std::thread;
fn main() {
    let mut v = vec![1];
    thread::scope(|s| { let h = s.spawn(|| v.push(1)); h.join().unwrap(); v.push(2); });
    println!("{:?}", v);
}
