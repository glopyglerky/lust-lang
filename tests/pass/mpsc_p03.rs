use std::sync::mpsc::sync_channel;
use std::thread;
fn main() {
    let (tx, rx) = sync_channel(0);
    let h = thread::spawn(move || { for i in 0..5 { tx.send(i).unwrap(); } });
    let mut s = 0;
    for _ in 0..5 { s += rx.recv().unwrap(); }
    h.join().unwrap();
    println!("{} {:?}", s, rx.recv());
    let (t2, r2) = sync_channel::<u8>(2);
    t2.send(1).unwrap(); t2.send(2).unwrap();
    println!("{:?} {:?} {:?}", r2.recv(), r2.try_recv(), r2.try_recv());
}
