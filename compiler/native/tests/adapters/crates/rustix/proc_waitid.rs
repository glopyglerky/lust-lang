// rustix 1.1.5 process::waitid: a NOWAIT wait leaves the child waitable
// for std's wait, NOHANG on a running child is Ok(None), and errors
// (no such child, a bad pid fd) come back as Errno.
use rustix::io::Errno;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
use std::process::Command;

fn main() {
    let opts = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
    println!("{} {}", opts.bits(), opts.contains(WaitIdOptions::NOWAIT));
    let mut child = Command::new("true").spawn().unwrap();
    let pid = Pid::from_child(&child);
    let r = waitid(WaitId::Pid(pid), opts);
    println!("{:?}", r.as_ref().map(|s| s.is_some()));
    println!("{:?}", child.wait().map(|s| s.success()));
    // Reaped: no such child now.
    println!("{:?}", waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).map(|s| s.is_some()));
    let mut slow = Command::new("sleep").arg("5").spawn().unwrap();
    let spid = Pid::from_child(&slow);
    let r = waitid(WaitId::Pid(spid), WaitIdOptions::EXITED | WaitIdOptions::NOHANG);
    println!("{:?}", r.map(|s| s.is_some()));
    let _ = slow.kill();
    let r = waitid(WaitId::Pid(spid), WaitIdOptions::EXITED);
    println!("{:?}", r.map(|s| s.is_some()));
    // No options: EINVAL.
    println!("{:?}", waitid(WaitId::All, WaitIdOptions::empty()).err());
    println!("{:?}", waitid(WaitId::Pgid(None), WaitIdOptions::EXITED | WaitIdOptions::NOHANG).err() == Some(Errno::CHILD));
    let id: WaitId<'static> = WaitId::Pgid(Some(Pid::INIT));
    println!("{:?} {:?} {:?}", id.clone(), WaitId::All, WaitId::Pgid(None));
    let _ = slow.wait();
}
