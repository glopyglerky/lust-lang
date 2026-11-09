// VecDeque's Index panics with std's `out of bounds access`.
use std::collections::VecDeque;

fn main() {
    let d: VecDeque<i32> = VecDeque::from(vec![1]);
    let i = d.len();
    println!("{}", d[i]);
}
