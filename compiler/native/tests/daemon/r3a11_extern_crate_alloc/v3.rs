extern crate alloc;
use alloc::vec::Vec;
fn main() {
    let mut v: Vec<u32> = Vec::new();
    v.push(4);
    println!("{}", v[0]);
}
