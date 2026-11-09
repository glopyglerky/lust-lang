// feature: ext.rustix
// form: the isprocessalive_unix.rs form (Pid, test_kill_process, Errno) plus a winsize error
// cargo-dep: rustix = { version = "=1.1.5", features = ["process", "termios"] }
use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};
fn main() {
    let alive = match Pid::from_raw(std::process::id() as i32) {
        Some(pid) => match test_kill_process(pid) {
            Ok(()) => true,
            Err(e) => e == Errno::PERM,
        },
        None => false,
    };
    println!("{alive}");
}
