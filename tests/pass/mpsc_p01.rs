use std::sync::mpsc;
use std::thread;
fn main() {
    let (tx, rx) = mpsc::channel();
    let mut hs = Vec::new();
    for i in 0..4u32 {
        let tx = tx.clone();
        hs.push(thread::spawn(move || { for j in 0..3 { tx.send(i * 10 + j).unwrap(); } }));
    }
    drop(tx);
    let mut got = Vec::new();
    while let Ok(v) = rx.recv() { got.push(v); }
    got.sort();
    for h in hs { h.join().unwrap(); }
    println!("{:?} {:?}", got, rx.try_recv());
}
