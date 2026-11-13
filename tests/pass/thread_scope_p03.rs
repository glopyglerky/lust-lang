use std::thread;
fn main() {
    let mut a = vec![1, 2];
    let mut b = vec![3];
    thread::scope(|s| {
        s.spawn(|| a.push(9));
        s.spawn(|| b.push(8));
    });
    println!("{:?} {:?}", a, b);
}
