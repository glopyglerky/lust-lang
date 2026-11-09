// expect: L0304
use std::thread;
fn main() {
    let mut v = vec![1];
    thread::scope(|s| {
        s.spawn(|| v.push(3));
        thread::scope(|s2| { s2.spawn(|| v.push(2)); });
    });
}
