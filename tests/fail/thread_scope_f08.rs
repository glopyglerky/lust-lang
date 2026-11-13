// expect: L0733
use std::thread;
fn main() {
    let mut out = None;
    thread::scope(|s| { out = Some(s.spawn(|| 3)); });
    println!("{:?}", out.unwrap().join());
}
