// expect: L0306
use std::thread;
fn main() {
    thread::scope(|s| {
        let local = vec![1];
        s.spawn(|| println!("{:?}", local));
    });
}
