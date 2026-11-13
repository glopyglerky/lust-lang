use std::thread;
fn main() {
    let mut v = vec![1, 2, 3];
    let total = thread::scope(|s| {
        let h = s.spawn(|| v.iter().sum::<i32>());
        let h2 = s.spawn(|| v.len());
        h.join().unwrap() + h2.join().unwrap() as i32
    });
    v.push(4);
    println!("{} {:?}", total, v);
}
