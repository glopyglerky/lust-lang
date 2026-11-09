// expect: L0115
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel::<u8>(); match rx.try_recv() { Err(mpsc::TryRecvError::Empty) => {}, Ok(_) => {} } drop(tx); }
