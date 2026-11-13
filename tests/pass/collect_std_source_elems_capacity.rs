// Collecting an iterator without an exact size into a Vec of std source element types (Cell,
// RefCell, Mutex) builds, with Rust's MIN_NON_ZERO_CAP growth (it failed to build before:
// "iterator expansion: element layout unknown").
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Mutex;

fn main() {
    let mut m: HashMap<u8, Cell<u32>> = HashMap::new();
    m.insert(1, Cell::new(5));
    let v: Vec<Cell<u32>> = m.into_values().collect();
    println!("{} {} {}", v.len(), v.capacity(), v[0].get());
    let c: Vec<Cell<u8>> = (0..3u8).filter(|i| *i != 1).map(|i| Cell::new(i)).collect();
    println!("{} {}", c.len(), c.capacity());
    let r: Vec<RefCell<Vec<u32>>> = (0..6u32).filter(|i| i % 2 == 0).map(|i| RefCell::new(vec![i])).collect();
    println!("{} {} {:?}", r.len(), r.capacity(), r[2].borrow());
    let l: Vec<Mutex<u16>> = (0..9u16).take_while(|i| *i < 5).map(|i| Mutex::new(i)).collect();
    println!("{} {} {}", l.len(), l.capacity(), l[4].lock().unwrap());
    let big: Vec<Cell<[u8; 1500]>> = (0..3u8).filter(|_| true).map(|i| Cell::new([i; 1500])).collect();
    println!("{} {}", big.len(), big.capacity());
    // In place: `Cell<u32>` has `u32`'s size and alignment, so the buffer is reused.
    let src: Vec<u32> = (0..5u32).collect();
    let cells: Vec<Cell<u32>> = src.into_iter().filter(|_| true).map(|i| Cell::new(i)).collect();
    println!("{} {} {}", cells.len(), cells.capacity(), std::mem::size_of::<Option<Cell<&u8>>>());
}
