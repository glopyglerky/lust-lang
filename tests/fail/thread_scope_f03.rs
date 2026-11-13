// expect: L0733
use std::thread;
fn main() {
    let h = thread::scope(|s| s.spawn(|| 1));
    println!("{:?}", h.join());
}
