// expect: L0110
// rustc E0308: upstream bitflags take `&self` (`bits(&self)`).
use nix::sys::fanotify::InitFlags;

fn main() {
    println!("{}", InitFlags::bits(InitFlags::FAN_CLOEXEC));
}
