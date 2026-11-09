// expect: L0303
// rustc E0502: `write!(s, "{}", s)` borrows s mutably and immutably.
use std::fmt::Write;
fn main() {
    let mut s = String::from("a");
    let _ = write!(s, "{}", s);
    println!("{}", s);
}
