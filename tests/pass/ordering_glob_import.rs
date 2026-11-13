// A glob of std::cmp brings Ordering.
use std::cmp::*;
fn f(a: i32, b: i32) -> Ordering {
    a.cmp(&b)
}
fn main() {
    match f(1, 2) {
        Ordering::Less => println!("lt"),
        o => println!("{:?}", o),
    }
}
