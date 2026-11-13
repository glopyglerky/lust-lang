// expect: L0303
// rustc E0502: a RawDirEntry borrows the RawDir mutably, so the
// RawDir cannot be used while the entry is alive.
use rustix::fs::{CWD, Mode, OFlags, RawDir, openat};
use std::mem::MaybeUninit;

fn main() {
    let fd = openat(CWD, ".", OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    let mut buf = [MaybeUninit::<u8>::uninit(); 4096];
    let mut raw = RawDir::new(&fd, &mut buf);
    let e = raw.next().unwrap().unwrap();
    println!("{}", raw.is_buffer_empty());
    println!("{:?}", e.file_name());
}
