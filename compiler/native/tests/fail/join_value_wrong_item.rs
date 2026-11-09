// expect: L0612
// rustc E0631: JoinHandle::join takes the handle by value.
use std::thread::JoinHandle;
fn main() { let hs: Vec<JoinHandle<u8>> = vec![std::thread::spawn(|| 1u8)]; let _r: Vec<_> = hs.iter().map(JoinHandle::join).collect(); }
