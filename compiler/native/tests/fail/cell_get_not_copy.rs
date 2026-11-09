// expect: L0612
use std::cell::Cell;
fn main() {
    let c = Cell::new(String::from("a"));
    println!("{}", c.get());
}
