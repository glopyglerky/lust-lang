// expect: L0303
// rustc E0502: an inotify Event borrows the Reader mutably.
use rustix::fs::inotify::{self, CreateFlags, Reader};
use std::mem::MaybeUninit;

fn main() {
    let ino = inotify::init(CreateFlags::NONBLOCK).unwrap();
    let mut buf = [MaybeUninit::<u8>::uninit(); 4096];
    let mut reader = Reader::new(&ino, &mut buf);
    let ev = reader.next();
    println!("{}", reader.is_buffer_empty());
    println!("{}", ev.is_ok());
}
