// panic: range end index 5 out of range for slice of length 3
use std::collections::VecDeque;
fn main() {
    let mut d: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    println!("before");
    let _ = d.drain(0..5);
    println!("after");
}
