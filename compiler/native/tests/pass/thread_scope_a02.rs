use std::thread;
fn main() {
    let v = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| { s.spawn(|| println!("{}", v.len())); });
    });
}
