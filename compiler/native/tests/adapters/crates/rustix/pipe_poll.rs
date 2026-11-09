// rustix 1.1.5 pipe, event, stdio, and termios: a pipe round
// trip, poll readiness and timeout, and the winsize error on a
// pipe stdout (the harness runs with piped output).
use rustix::event::{PollFd, PollFlags, poll};
use rustix::fs::Timespec;
use rustix::pipe::{PipeFlags, pipe_with};

fn main() {
    let (read, write) = pipe_with(PipeFlags::CLOEXEC).unwrap();
    let zero = Timespec { tv_sec: 0, tv_nsec: 0 };
    let mut fds = [PollFd::new(&read, PollFlags::IN)];
    println!("{}", poll(&mut fds, Some(&zero)).unwrap());
    println!("{}", rustix::io::write(&write, b"abc").unwrap());
    println!("{}", poll(&mut fds, Some(&zero)).unwrap());
    println!("{}", fds[0].revents().contains(PollFlags::IN));
    let mut buf = [0u8; 8];
    let n = rustix::io::read(&read, &mut buf).unwrap();
    println!("{} {}", n, String::from_utf8_lossy(&buf[..n]));
    drop(write);
    let n2 = rustix::io::read(&read, &mut buf).unwrap();
    println!("{n2}");
    let st = rustix::fs::fstat(&read).unwrap();
    println!("{:?} {}", rustix::fs::FileType::from_raw_mode(st.st_mode), st.st_ino > 0);
    match rustix::termios::tcgetwinsize(rustix::stdio::stdout()) {
        Ok(ws) => println!("winsize {} {}", ws.ws_row, ws.ws_col),
        Err(e) => println!("winsize err {e}"),
    }
    println!("{}", rustix::stdio::stdin().as_raw_fd());
}

use rustix::fd::AsRawFd as _;
