// expect: L0311
// rustc E0505: `r` borrows `s` through the closure's output.
use std::cell::Cell;
thread_local! {
    static L: Cell<u8> = const { Cell::new(0) };
}
fn main() {
    let s = String::from("q");
    let r = L.with(|_l| s.as_str());
    drop(s);
    println!("{}", r);
}
