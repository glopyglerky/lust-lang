// expect: L0510
// rustc E0616: the storage of a std collection is private.
use std::collections::VecDeque;
fn main() {
    let q: VecDeque<u8> = VecDeque::new();
    println!("{}", q.buf.len());
}
