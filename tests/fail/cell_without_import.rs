// expect: L0102
fn main() {
    let c = std::cell::Cell::new(1);
    let d = Cell::new(2);
    println!("{} {}", c.get(), d.get());
}
