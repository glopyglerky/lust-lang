// expect: L0110
// rustc E0308: recv_timeout takes a Duration, not an integer.
use std::sync::mpsc;
fn main() { let (_tx, rx) = mpsc::channel::<u8>(); let _ = rx.recv_timeout(5); }
