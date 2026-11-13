// rustix 1.1.5 fs::inotify (init, add_watch, remove_watch;
// events read with io::read as fswatch/unix.rs does) and
// fs::statvfs/fstatvfs. Prints only values this program makes
// itself or comparisons between two calls on the same files.
use rustix::fs::inotify::{self, CreateFlags, WatchFlags};
use rustix::fs::{fstatvfs, statvfs, Mode, OFlags};

fn le32(b: &[u8], off: usize) -> u32 {
    (b[off] as u32) | ((b[off + 1] as u32) << 8) | ((b[off + 2] as u32) << 16) | ((b[off + 3] as u32) << 24)
}

fn main() {
    let dir = std::env::temp_dir().join(format!("lust-rustix-ino-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let ino = inotify::init(CreateFlags::NONBLOCK | CreateFlags::CLOEXEC).unwrap();
    let wd = inotify::add_watch(&ino, &dir, WatchFlags::CREATE | WatchFlags::DELETE).unwrap();
    println!("{}", wd >= 0);
    std::fs::write(dir.join("new.txt"), b"x").unwrap();

    let mut pollfds = [rustix::event::PollFd::new(&ino, rustix::event::PollFlags::IN)];
    let ts = rustix::event::Timespec { tv_sec: 5, tv_nsec: 0 };
    println!("{}", rustix::event::poll(&mut pollfds, Some(&ts)).unwrap());

    let mut buf = [0u8; 4096];
    let n = rustix::io::read(&ino, &mut buf).unwrap();
    let buf = &buf[..n];
    let ev_wd = le32(buf, 0) as i32;
    let mask = le32(buf, 4);
    let len = le32(buf, 12) as usize;
    let name = &buf[16..16 + len];
    let end = name.iter().position(|&b| b == 0).unwrap_or(len);
    println!("{} {} {}", ev_wd == wd, mask & WatchFlags::CREATE.bits() != 0, String::from_utf8_lossy(&name[..end]));

    inotify::remove_watch(&ino, wd).unwrap();
    println!("{:?}", inotify::add_watch(&ino, dir.join("missing"), WatchFlags::CREATE).unwrap_err() == rustix::io::Errno::NOENT);
    println!("{}", inotify::remove_watch(&ino, wd).is_err());
    println!("{} {}", CreateFlags::NONBLOCK.bits(), WatchFlags::ALL_EVENTS.bits());

    let a = statvfs(&dir).unwrap();
    let dfd = rustix::fs::open(&dir, OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    let b = fstatvfs(&dfd).unwrap();
    println!(
        "{} {} {} {} {}",
        a.f_bsize == b.f_bsize && a.f_bsize > 0,
        a.f_blocks == b.f_blocks && a.f_blocks > 0,
        a.f_fsid == b.f_fsid,
        a.f_namemax == b.f_namemax && a.f_namemax > 0,
        a.f_flag == b.f_flag && !a.f_flag.contains(rustix::fs::StatVfsMountFlags::RDONLY)
    );
    println!("{}", matches!(statvfs(dir.join("missing")), Err(e) if e == rustix::io::Errno::NOENT));
    std::fs::remove_dir_all(&dir).unwrap();
}
