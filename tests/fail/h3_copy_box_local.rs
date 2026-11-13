// expect: L0110
// rustc E0184: Box has a destructor, so Box<Local> cannot be Copy.
#[derive(Clone, Copy)]
struct Local;
impl Copy for Box<Local> {}
fn main() { println!("ok"); }
