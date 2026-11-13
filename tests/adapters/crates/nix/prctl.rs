// nix 0.31.3: the THP prctl pair thp_guard.rs writes, in a child
// thread so the flag change stays in this program, and Errno's
// text forms. get/set round trips print, never a machine value.
use nix::errno::Errno;
use nix::sys::prctl::{get_thp_disable, set_thp_disable};

fn main() {
    let before = get_thp_disable().unwrap();
    set_thp_disable(!before).unwrap();
    let mid = get_thp_disable().unwrap();
    set_thp_disable(before).unwrap();
    let after = get_thp_disable().unwrap();
    println!("{} {}", mid == !before, after == before);
    println!("{} {:?} {:?}", Errno::EPERM, Errno::EINVAL, Errno::from_raw(2));
    let e: nix::Error = Errno::EACCES; println!("{}", e.desc());
    let io: std::io::Error = Errno::ENOENT.into();
    println!("{:?}", io.kind());
}
