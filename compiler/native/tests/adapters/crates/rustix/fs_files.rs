// rustix 1.1.5 fs: open/openat, fstat/statat/statx, accessat,
// utimensat, mknodat, fcntl_getfl, and the flag types, in the
// forms osvfs.rs and compile.rs write them. Times print only
// the exact values this program sets itself.
use rustix::fs::{Access, AtFlags, CWD, FileType, Mode, OFlags, StatxFlags, Timespec, Timestamps, UTIME_OMIT};
use rustix::fs::{accessat, fcntl_getfl, fstat, mknodat, open, openat, statat, statx, utimensat};
use rustix::io::Errno;

fn main() {
    let dir = std::env::temp_dir().join(format!("lust-rustix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.txt");
    let fd = open(&file, OFlags::CREATE | OFlags::WRONLY | OFlags::CLOEXEC, Mode::from_raw_mode(0o644)).unwrap();
    println!("{}", rustix::io::write(&fd, b"hello rustix").unwrap());
    let st = fstat(&fd).unwrap();
    println!("{} {:?} {}", st.st_size, FileType::from_raw_mode(st.st_mode), Mode::from_raw_mode(st.st_mode).bits());
    drop(fd);
    let st2 = statat(CWD, &file, AtFlags::empty()).unwrap();
    println!("{} {}", st2.st_size, st2.st_ino == st.st_ino && st2.st_dev == st.st_dev);
    let wanted = StatxFlags::TYPE | StatxFlags::MTIME | StatxFlags::SIZE;
    let sx = statx(CWD, &file, AtFlags::SYMLINK_NOFOLLOW, wanted).unwrap();
    println!("{} {} {:?}", sx.stx_mask & wanted.bits() == wanted.bits(), sx.stx_size, FileType::from_raw_mode(u32::from(sx.stx_mode)));
    let times = Timestamps {
        last_access: Timespec { tv_sec: 1_600_000_000, tv_nsec: 123_456_789 },
        last_modification: Timespec { tv_sec: 0, tv_nsec: UTIME_OMIT },
    };
    utimensat(CWD, &file, &times, AtFlags::empty()).unwrap();
    let st3 = statat(CWD, &file, AtFlags::empty()).unwrap();
    println!("{} {} {}", st3.st_atime, st3.st_atime_nsec, st3.st_mtime == st.st_mtime && st3.st_mtime_nsec == st.st_mtime_nsec);
    println!("{:?} {:?}", accessat(CWD, &file, Access::EXISTS, AtFlags::EACCESS), accessat(CWD, &file, Access::EXEC_OK, AtFlags::EACCESS));
    let fifo = dir.join("fifo");
    mknodat(CWD, &fifo, FileType::Fifo, Mode::from_raw_mode(0o644), 0).unwrap();
    let fst = statat(CWD, &fifo, AtFlags::empty()).unwrap();
    println!("{:?}", FileType::from_raw_mode(fst.st_mode));
    let dfd = openat(CWD, &dir, OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC, Mode::empty()).unwrap();
    println!("{}", fcntl_getfl(&dfd).unwrap().contains(OFlags::NONBLOCK));
    let rfd = openat(&dfd, "f.txt", OFlags::RDONLY, Mode::empty()).unwrap();
    let mut buf = [0u8; 32];
    let n = rustix::io::read(&rfd, &mut buf).unwrap();
    println!("{} {}", n, String::from_utf8_lossy(&buf[..n]));
    println!("{}", open(dir.join("missing"), OFlags::RDONLY, Mode::empty()).unwrap_err());
    println!("{:?}", statat(CWD, &dir.join("missing"), AtFlags::empty()).unwrap_err() == Errno::NOENT);
    println!("{}", OFlags::PATH.bits());
    std::fs::remove_dir_all(&dir).unwrap();
}
