// capability: rejected: a Cell, RefCell, or Mutex of `str` or `[T]`
// expect: L0641

use std::sync::Mutex;

fn main() {
    let m: Box<Mutex<[u32]>> = Box::new(Mutex::new([1u32, 2]));
    println!("{}", m.lock().unwrap().len());
}
