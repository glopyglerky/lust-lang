// expect: L0104
// rustc E0599: upstream `PollFd` has no `events` accessor.
use rustix::event::{PollFd, PollFlags};

fn main() {
    let fd = rustix::stdio::stdin();
    let p = PollFd::new(&fd, PollFlags::IN);
    println!("{}", p.events().bits());
}
