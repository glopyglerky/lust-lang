use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn producer(tx: Sender<D>, n: u32) { for i in 0..n { tx.send(D(i)).unwrap(); } }
fn main() {
    let (tx, rx): (Sender<D>, Receiver<D>) = channel();
    producer(tx.clone(), 3);
    let first = rx.recv().unwrap();
    println!("got {}", first.0);
    drop(rx);
    println!("rx gone {}", tx.send(D(9)).is_err());
    let h = thread::spawn(move || drop(tx));
    h.join().unwrap();
    println!("{:?}", first.0);
}
