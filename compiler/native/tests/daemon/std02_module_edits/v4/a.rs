// A type error next to a std module use.
use std::collections::VecDeque;

pub fn go() {
    let mut q: VecDeque<u8> = VecDeque::new();
    q.push_back("x");
    println!("{}", q.len());
}
