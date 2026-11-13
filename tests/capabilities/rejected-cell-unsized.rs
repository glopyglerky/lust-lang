// capability: rejected: a Cell, RefCell, or Mutex of `str` or `[T]`
// expect: L0641

use std::cell::RefCell;

fn len(x: &RefCell<[u8]>) -> usize {
    x.borrow().len()
}

fn main() {
    let c = RefCell::new([1u8, 2]);
    println!("{}", len(&c));
}
