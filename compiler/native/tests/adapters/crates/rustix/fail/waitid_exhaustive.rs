// expect: L0115
// rustc E0004: upstream `WaitId` is `#[non_exhaustive]`.
use rustix::process::WaitId;

fn n(w: WaitId<'_>) -> u32 {
    match w {
        WaitId::All => 0,
        WaitId::Pid(_) => 1,
        WaitId::Pgid(_) => 2,
        WaitId::PidFd(_) => 3,
    }
}

fn main() {
    println!("{}", n(WaitId::All));
}
