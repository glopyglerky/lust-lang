// Receiver::recv_timeout and RecvTimeoutError (tf-p6 R09): queued value, then
// disconnection, then the deadline; Display/Debug/type_name; path and value forms;
// a rendezvous channel pairs with a blocked sender.
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
fn main() {
    let (tx, rx) = mpsc::channel::<String>();
    let t0 = Instant::now();
    let r = rx.recv_timeout(Duration::from_millis(50));
    println!("{:?} {}", r, t0.elapsed() >= Duration::from_millis(50));
    tx.send("a".to_string()).unwrap();
    println!("{:?}", rx.recv_timeout(Duration::ZERO));
    
    let h = std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(30)); tx.send("b".into()).unwrap(); });
    println!("{:?}", rx.recv_timeout(Duration::from_secs(5)));
    h.join().unwrap();
    match rx.recv_timeout(Duration::from_secs(5)) { Err(RecvTimeoutError::Disconnected) => println!("disc"), _ => println!("?") }
    println!("{} | {} | {:?} {:?}", RecvTimeoutError::Timeout, mpsc::RecvTimeoutError::Disconnected, RecvTimeoutError::Timeout, RecvTimeoutError::Timeout == RecvTimeoutError::Disconnected);
    println!("{}", std::any::type_name::<RecvTimeoutError>());
    // The sender blocks in send until a receive takes the value; a long
    // timeout (not a sleep and try_recv) keeps this exact under load.
    let (stx, srx) = mpsc::sync_channel::<u8>(0);
    let h = std::thread::spawn(move || { stx.send(7).unwrap(); stx.send(8).unwrap(); });
    println!("{:?}", srx.recv_timeout(Duration::from_secs(30)));
    println!("{:?}", srx.recv_timeout(Duration::from_secs(30)));
    h.join().unwrap();
    println!("{:?} {:?}", srx.try_recv(), srx.recv_timeout(Duration::ZERO));
    println!("{:?}", Receiver_path());
}
#[allow(non_snake_case)]
fn Receiver_path() -> Result<u8, RecvTimeoutError> { let (_t, r) = mpsc::channel::<u8>(); mpsc::Receiver::recv_timeout(&r, Duration::from_nanos(1)) }
