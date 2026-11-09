// nix 0.31.3 sys::signal: the Signal enum's numbers, derives and glob
// export, and SigSet's set forms and thread mask calls. The mask is read
// back from /proc/thread-self/status (SigBlk), so only this thread's own
// changes print.
use nix::sys::signal::{SIGINT, SIGUSR1, SigSet, Signal};
use std::collections::BTreeSet;

fn blocked() -> u64 {
    let s = std::fs::read_to_string("/proc/thread-self/status").unwrap();
    let line = s.lines().find(|l| l.starts_with("SigBlk:")).unwrap();
    u64::from_str_radix(line[7..].trim(), 16).unwrap()
}

fn main() {
    let all = [Signal::SIGHUP, Signal::SIGSTKFLT, Signal::SIGIO, Signal::SIGPWR, Signal::SIGSYS, SIGINT];
    for s in all {
        print!("{:?}={} ", s, s as i32);
    }
    println!();
    println!("{} {:?} {:?}", Signal::SIGTERM > Signal::SIGKILL, Signal::SIGURG.cmp(&Signal::SIGURG), Signal::SIGCHLD.max(Signal::SIGALRM));
    let set: BTreeSet<Signal> = [Signal::SIGWINCH, SIGINT, SIGINT].into_iter().collect();
    println!("{:?}", set);
    let mut s: SigSet = [Signal::SIGUSR2, Signal::SIGTERM].into_iter().collect();
    s.add(SIGUSR1);
    s.remove(Signal::SIGTERM);
    println!("{} {} {}", s.contains(SIGUSR1), s.contains(Signal::SIGTERM), s.contains(Signal::SIGUSR2));
    s.extend([Signal::SIGPROF]);
    let t = s;
    println!("{}", t.contains(Signal::SIGPROF));
    let before = blocked();
    println!("{:?}", s.thread_block());
    println!("{:x}", blocked() & !before);
    println!("{:?}", s.thread_unblock());
    println!("{}", blocked() == before & !(1 << 9 | 1 << 11 | 1 << 26));
    // Unblocking what is not blocked, and the always-unblockable SIGKILL.
    let k: SigSet = [Signal::SIGKILL, Signal::SIGSTOP].into_iter().collect();
    println!("{:?} {:?}", k.thread_unblock(), k.thread_block());
    println!("{}", blocked() & (1 << 8 | 1 << 18));
    s.clear();
    println!("{}", s.contains(SIGUSR1));
    let e = SigSet::empty();
    println!("{:?}", e.thread_unblock());
}
