// expect: L0304
use std::thread;
fn main() {
    let mut v = vec![1];
    thread::scope(|s| { s.spawn(|| v.push(1)); s.spawn(|| v.push(2)); });
    println!("{:?}", v);
}
