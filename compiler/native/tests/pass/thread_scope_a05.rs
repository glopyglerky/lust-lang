use std::thread;
fn main() {
    let mut v = vec![1];
    thread::scope(|s| {
        thread::scope(|s2| { s2.spawn(|| v.push(2)); });
        s.spawn(|| v.push(3));
    });
    v.sort();
    println!("{:?}", v);
}
