struct Cell {
    v: u8,
}
impl Cell {
    fn new(v: u8) -> Cell {
        Cell { v }
    }
}
fn main() {
    let c = Cell::new(4);
    println!("{}", c.v);
}
