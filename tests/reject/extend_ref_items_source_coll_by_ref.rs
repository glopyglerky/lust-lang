// expect: L9001
// Valid Rust. A later push of a value picks VecDeque's Extend<&T>, which lust's
// library source does not write yet (method selection cannot pick between two
// `Extend` impls of one type).
use std::collections::VecDeque;
fn main() {
    let s = 5u8;
    let mut v = VecDeque::new();
    v.extend([&s]);
    v.push_back(1u8);
    println!("{:?}", v);
}
