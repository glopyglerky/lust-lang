// expect: L0612
use std::thread;
use std::rc::Rc;
fn main() {
    let r = Rc::new(1);
    thread::scope(|s| { s.spawn(|| println!("{}", r)); });
}
