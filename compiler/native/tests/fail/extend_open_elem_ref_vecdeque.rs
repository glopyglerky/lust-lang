// expect: L0610
// rustc E0282: VecDeque has Extend<T> and Extend<&T> (T: Copy), so nothing picks T.
use std::collections::VecDeque;
fn main() {
    let s = 5u8;
    let mut v = VecDeque::new();
    v.extend([&s]);
    println!("{}", v.len());
}
