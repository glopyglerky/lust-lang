// Variants in both namespaces, kept apart by explicit imports and
// renames: rustc accepts every line here.
enum E {
    B,
    C(u8),
}
mod b {
    pub struct B;
    impl B {
        pub fn v() -> u8 {
            3
        }
    }
}
use b::*;
use E::B as X;
use std::cmp::Ordering::*;
fn takes(_x: B) -> u8 {
    1
}
fn main() {
    let e = X;
    let o = Less;
    match e {
        X => println!("{} {:?} {}", B::v(), o, takes(B)),
        _ => println!("no"),
    }
}
