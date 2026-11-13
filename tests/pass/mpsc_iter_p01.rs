use std::sync::mpsc;
use std::thread;
fn main() {
    let (tx, rx) = mpsc::channel();
    let hs: Vec<_> = (0..3).map(|i| { let tx = tx.clone(); thread::spawn(move || { for j in 0..4 { tx.send(i * 10 + j).unwrap(); } }) }).collect();
    drop(tx);
    let mut got: Vec<i32> = Vec::new();
    for v in rx { got.push(v); }
    for h in hs { h.join().unwrap(); }
    got.sort();
    println!("{:?}", got);
}
