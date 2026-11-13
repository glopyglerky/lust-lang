use std::sync::mpsc::{self, TryRecvError, RecvError};
fn main() {
    let (tx, rx) = mpsc::channel::<String>();
    println!("{:?}", rx.try_recv());
    tx.send(String::from("a")).unwrap();
    println!("{:?} {:?}", rx.try_recv(), rx.try_recv() == Err(TryRecvError::Empty));
    drop(tx);
    println!("{:?} {:?}", rx.recv(), rx.recv() == Err(RecvError));
    match rx.try_recv() { Err(TryRecvError::Disconnected) => println!("disc"), Err(TryRecvError::Empty) => println!("empty"), Ok(s) => println!("{s}") }
    let (tx2, rx2) = mpsc::channel::<Vec<u8>>();
    drop(rx2);
    let e = tx2.send(vec![1, 2]);
    println!("{:?} {}", e, e.is_err());
    if let Err(err) = e { println!("{} {:?}", err, err); }
}
