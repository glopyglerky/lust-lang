// expect: L0612
use std::thread;
use std::cell::Cell;
fn main() {
    let c = Cell::new(1);
    thread::scope(|s| { s.spawn(|| c.set(2)); });
    println!("{}", c.get());
}
