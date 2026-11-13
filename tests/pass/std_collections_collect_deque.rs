// Collect into a VecDeque (W-STD-C phase 2): the compiler's Vec
// collect, wrapped; capacity follows Vec's rules.
use std::collections::VecDeque;
fn main() {
    let s: VecDeque<u8> = vec![1u8].into_iter().collect();
    println!("{}", s.len());
}
