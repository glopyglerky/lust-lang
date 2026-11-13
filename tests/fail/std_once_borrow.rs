// expect: L0311
// rustc E0505: the value from `get_or_init` borrows the cell.
use std::cell::OnceCell;
fn main() {
    let c = OnceCell::new();
    let r = c.get_or_init(|| String::from("a"));
    let d = c;
    println!("{}", r);
    drop(d);
}
