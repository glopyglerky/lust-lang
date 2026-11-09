// try_recv on a sync_channel(0) takes the value of a blocked sender, as std's
// zero-capacity channel does (was Err(Empty), then a hang in join).
use std::sync::mpsc;
use std::time::Duration;
fn main() {
    let (tx, rx) = mpsc::sync_channel::<u8>(0);
    let h = std::thread::spawn(move || { tx.send(7).unwrap(); });
    std::thread::sleep(Duration::from_millis(200));
    println!("{:?}", rx.try_recv());
    h.join().unwrap();
    println!("{}", mpsc::TryRecvError::Disconnected);
    println!("{}", mpsc::TryRecvError::Empty);
}
