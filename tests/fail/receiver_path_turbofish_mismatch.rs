// expect: L0110
// rustc E0308: `Receiver::<u16>::recv_timeout` takes a `&Receiver<u16>`.
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
fn main() { let (_tx, rx) = mpsc::channel::<u8>(); let _ = Receiver::<u16>::recv_timeout(&rx, Duration::ZERO); }
