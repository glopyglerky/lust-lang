use std::sync::mpsc::{channel, sync_channel};
use std::thread;
fn main() {
    let (tx, rx) = channel::<String>();
    let h = thread::spawn(move || { for s in ["a", "bb", "ccc"] { tx.send(s.to_string()).unwrap(); } });
    let mut n = 0;
    for s in &rx { n += s.len(); }
    h.join().unwrap();
    println!("{} {:?}", n, rx.try_recv());
    let (t2, r2) = sync_channel(1);
    let h2 = thread::spawn(move || { for i in 0..5u64 { t2.send(i * i).unwrap(); } });
    let total: u64 = r2.iter().sum();
    h2.join().unwrap();
    println!("{}", total);
    let (t3, r3) = channel();
    t3.send(1).unwrap(); t3.send(2).unwrap();
    let now: Vec<i32> = r3.try_iter().collect();
    println!("{:?} {:?}", now, r3.try_iter().next());
    drop(t3);
    let v: Vec<i32> = r3.into_iter().map(|x| x * 2).collect();
    println!("{:?}", v);
}
