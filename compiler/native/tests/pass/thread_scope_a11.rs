use std::thread;
fn main() {
    let mut chunks = vec![vec![1, 2], vec![3], vec![4, 5, 6]];
    thread::scope(|s| {
        for c in chunks.iter_mut() { s.spawn(move || c.push(c.len() as i32)); }
    });
    println!("{:?}", chunks);
}
