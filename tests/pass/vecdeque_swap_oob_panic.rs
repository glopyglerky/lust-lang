// VecDeque's swap asserts each index as std does.
use std::collections::VecDeque;

fn main() {
    let mut d: VecDeque<i32> = VecDeque::from(vec![1]);
    d.swap(0, 1);
    println!("ran");
}
