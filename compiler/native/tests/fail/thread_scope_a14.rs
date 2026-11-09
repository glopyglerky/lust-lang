// expect: L0301
use std::thread;
fn main() {
    let s = String::from("hi");
    thread::scope(|sc| { sc.spawn(move || println!("{}", s)); });
    println!("{}", s);
}
