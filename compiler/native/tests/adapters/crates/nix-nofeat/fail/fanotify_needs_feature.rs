// expect: L0102
// rustc E0433: `sys::fanotify` is behind upstream's `fanotify` feature.
fn main() {
    let f = nix::sys::fanotify::InitFlags::FAN_CLOEXEC;
    let _ = f;
    println!("x");
}
