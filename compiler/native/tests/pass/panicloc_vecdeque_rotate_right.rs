// VecDeque::rotate_right asserts in std's vec_deque/mod.rs (tf-panicloc).
use std::collections::VecDeque;
fn main() {
    let mut d: VecDeque<u8> = (1..4).collect();
    d.rotate_right(1);
    println!("{:?}", d);
    let n = std::env::args().count();
    d.rotate_right(n + 3);
}
