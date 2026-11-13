// expect: L0303
use std::thread;
fn main() {
    let mut chunks = vec![vec![1, 2], vec![3]];
    thread::scope(|s| {
        for c in chunks.iter_mut() { s.spawn(move || c.push(0)); }
        println!("{:?}", chunks);
    });
}
