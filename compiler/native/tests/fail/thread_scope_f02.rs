// expect: L0303
use std::thread;
fn main() {
    let mut v = vec![1];
    thread::scope(|s| { s.spawn(|| v.push(1)); println!("{}", v.len()); });
}
