// expect: L0110
// A std path names std's Cell, not the user type of the same name.
struct Cell(i32);
fn main() {
    let c: std::cell::Cell<i32> = Cell(1);
    println!("{}", c.get());
}
