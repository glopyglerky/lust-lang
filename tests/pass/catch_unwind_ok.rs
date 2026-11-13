// catch_unwind of a closure that does not panic is Ok in both modes.
use std::panic;
fn main() {
    let v = vec![1, 2];
    let s = String::from("x");
    let r = panic::catch_unwind(|| v.len() + s.len());
    println!("{:?}", r);
    let r = panic::catch_unwind(move || { let w = v; w.len() });
    println!("{:?}", r);
}
