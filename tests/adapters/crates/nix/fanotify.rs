// nix 0.31.3 sys::fanotify (feature fanotify) in the forms
// fswatch/unix.rs writes, plus the Errno enum cast it does.
// The consumer flags are the FID reporting mode, which the
// kernel grants without CAP_SYS_ADMIN (plain notification
// groups get EPERM here), so the mark path runs on this box
// and on boat; the Err branch prints the errno form if a
// kernel refuses. Flag bits are the Linux
// values, including the FAN_REPORT_* bits nix has no names for
// (they pass through from_bits_retain, as ts-rust does).
use nix::errno::Errno;
use nix::sys::fanotify::{EventFFlags, Fanotify, InitFlags, MarkFlags, MaskFlags};
use std::os::fd::{AsFd, AsRawFd};

fn main() {
    println!("{} {} {}", Errno::EINVAL as i32, Errno::from_raw(11) == Errno::EWOULDBLOCK, Errno::from_raw(9999) == Errno::UnknownErrno);
    println!("{} {:?}", Errno::ENOSPC, Errno::EOWNERDEAD);
    println!("{}", Errno::UnknownErrno.desc());

    let init = InitFlags::from_bits_retain(0x0000_0001 | 0x0000_0002 | 0x0000_0200 | 0x0000_0c00);
    let eff = EventFFlags::from_bits_retain(0o2000000);
    println!("{} {} {}", init.bits(), eff.bits(), nix::sys::fanotify::FANOTIFY_METADATA_VERSION);
    let mask = MaskFlags::FAN_CREATE | MaskFlags::FAN_DELETE | MaskFlags::FAN_ONDIR | MaskFlags::FAN_EVENT_ON_CHILD;
    let mark = MarkFlags::FAN_MARK_ADD | MarkFlags::FAN_MARK_ONLYDIR | MarkFlags::FAN_MARK_DONT_FOLLOW;
    println!("{} {}", mask.bits(), mark.bits());

    match Fanotify::init(init, eff) {
        Err(e) => println!("init err {} {}", e as i32, e),
        Ok(f) => {
            let dir = std::env::temp_dir();
            let dirfd = std::fs::File::open(&dir).unwrap();
            let r = f.mark(mark, mask, &dirfd, Some(dir.as_path()));
            println!("init ok {} {}", f.as_fd().as_raw_fd() >= 0, r.is_ok());
        }
    }
}
