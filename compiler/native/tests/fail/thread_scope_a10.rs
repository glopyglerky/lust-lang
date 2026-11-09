// expect: L0733
use std::thread;
fn main() {
    let v = vec![1];
    let s2 = thread::scope(|s| s);
}
