// expect: L0115
// rustc E0004: `Signal` is `#[non_exhaustive]` upstream, so a match outside nix needs `_`.
use nix::sys::signal::Signal;

fn main() {
    let s = Signal::SIGINT;
    let n = match s {
        Signal::SIGHUP | Signal::SIGINT | Signal::SIGQUIT | Signal::SIGILL | Signal::SIGTRAP | Signal::SIGABRT | Signal::SIGBUS => 1,
        Signal::SIGFPE | Signal::SIGKILL | Signal::SIGUSR1 | Signal::SIGSEGV | Signal::SIGUSR2 | Signal::SIGPIPE | Signal::SIGALRM => 2,
        Signal::SIGTERM | Signal::SIGSTKFLT | Signal::SIGCHLD | Signal::SIGCONT | Signal::SIGSTOP | Signal::SIGTSTP | Signal::SIGTTIN => 3,
        Signal::SIGTTOU | Signal::SIGURG | Signal::SIGXCPU | Signal::SIGXFSZ | Signal::SIGVTALRM | Signal::SIGPROF | Signal::SIGWINCH => 4,
        Signal::SIGIO | Signal::SIGPWR | Signal::SIGSYS => 5,
    };
    println!("{}", n);
}
