use std::thread;
use std::sync::Mutex;
fn main() {
    let m = Mutex::new(Vec::new());
    let data = vec![10, 20, 30];
    thread::scope(|s| {
        for (i, d) in data.iter().enumerate() {
            let m = &m;
            s.spawn(move || { m.lock().unwrap().push(i * 100 + *d); });
        }
    });
    let mut r = m.into_inner().unwrap(); r.sort();
    println!("{:?}", r);
}
