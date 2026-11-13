// VecDeque's split_off asserts `at` with std's message.
use std::collections::VecDeque;

fn main() {
    let mut d: VecDeque<i32> = VecDeque::from(vec![1]);
    let _ = d.split_off(2);
    println!("ran");
}
