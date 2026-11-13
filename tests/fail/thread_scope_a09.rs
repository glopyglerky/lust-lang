// expect: L0303
use std::thread;
fn main() {
    let mut x = 0;
    thread::scope(|s| { let h = s.spawn(|| { x += 1; }); drop(h); x += 1; });
}
