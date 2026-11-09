// rustix 1.1.5 process + io::Errno: pids, kill probing, signals,
// rlimits, and the error texts. Only relationships print, never a
// pid or a limit value another machine would change.
use rustix::io::Errno;
use rustix::process::{Pid, Resource, Rlimit, Signal, getpid, getppid, getrlimit, setrlimit, test_kill_process};

fn main() {
    let own = getpid();
    println!("{} {}", own.as_raw_pid() == std::process::id() as i32, own.is_init());
    println!("{} {}", getppid().is_some(), getppid() != Some(own));
    println!("{:?} {:?} {}", Pid::from_raw(0), Pid::from_raw(1), Pid::INIT);
    println!("{} {}", Pid::as_raw(Some(own)) == own.as_raw_pid(), Pid::as_raw(None));
    println!("{:?}", test_kill_process(own));
    let missing = Pid::from_raw(i32::MAX).unwrap();
    println!("{}", test_kill_process(missing).unwrap_err());
    println!("{:?}", test_kill_process(missing).unwrap_err());
    println!("{:?} {:?} {:?} {:?}", Signal::TERM, Signal::from_named_raw(9), Signal::from_named_raw(0), Signal::from_named_raw(64));
    println!("{} {}", Signal::INT.as_raw(), Errno::NOENT.raw_os_error());
    let io_err = std::io::Error::from(Errno::PERM);
    println!("{} {:?} {:?}", io_err, io_err.kind(), Errno::from_io_error(&io_err));
    println!("{:?} {:?}", Resource::Nofile, Resource::As);
    let limit = getrlimit(Resource::Nofile);
    println!("{} {}", limit.current.is_some(), limit.maximum.is_some());
    let same = Rlimit { current: limit.current, maximum: limit.maximum };
    println!("{:?} {}", setrlimit(Resource::Nofile, same), getrlimit(Resource::Nofile) == limit);
    let stack = getrlimit(Resource::Stack);
    println!("{}", stack.current.map_or(false, |v| v >= 1));
}
