// expect: L0102
// A glob in a block ends with the block.
fn main() {
    {
        use std::cell::*;
        let c = Cell::new(1);
        println!("{}", c.get());
    }
    let d = Cell::new(2);
    println!("{}", d.get());
}
