// `Cell::take` and `RefCell::take` by path and as values (ts-rust
// store.rs `try_with(Cell::take)`): `T::default()` left behind.
use std::cell::{Cell, RefCell};
thread_local! { static T: Cell<Vec<u8>> = const { Cell::new(Vec::new()) }; }
fn main() {
    T.with(|c| c.set(vec![1, 2]));
    let v = T.try_with(Cell::take).unwrap();
    println!("{:?} {:?}", v, T.with(Cell::take));
    let cells = [Cell::new(3u8), Cell::new(4)];
    let got: Vec<u8> = cells.iter().map(Cell::take).collect();
    println!("{:?} {}", got, cells[0].get());
    let r = RefCell::new(String::from("x"));
    let s = RefCell::take(&r);
    println!("{:?} {:?}", s, r);
    let c = Cell::new(Some(5));
    println!("{:?} {:?}", Cell::take(&c), c.get());
}
