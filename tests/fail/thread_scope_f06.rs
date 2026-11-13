// expect: L0306
use std::thread;
fn main() {
    thread::scope(|s| { let x = 5; s.spawn(|| println!("{}", x)); });
}
